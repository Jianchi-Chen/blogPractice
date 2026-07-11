use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    auth::{JwtAuth, MaybeJwtAuth, is_admin, require_admin},
    db::AppState,
    error::{AppError, AppResult},
    models::article::{
        Article, ArticleInput, ArticleListQuery, ArticleStatusInput, ArticleSummary,
    },
    repositories::article::{
        create_article, delete_article_by_id, find_article_by_id, list_articles, update_article,
        update_article_status,
    },
};

#[derive(Serialize, Debug)]
pub struct ArticleListResponse {
    pub articles: Vec<ArticleSummary>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    MaybeJwtAuth(auth): MaybeJwtAuth,
    Query(query): Query<ArticleListQuery>,
) -> AppResult<Json<ArticleListResponse>> {
    let include_unpublished = match auth {
        Some(claims) => is_admin(&state, &claims.user_id).await?,
        None => false,
    };
    let articles = list_articles(&state.pool, query, include_unpublished).await?;
    Ok(Json(ArticleListResponse { articles }))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Json(input): Json<ArticleInput>,
) -> AppResult<(StatusCode, Json<Article>)> {
    require_admin(&state, &claims).await?;
    validate_article_input(&input)?;
    let article = create_article(&state.pool, &input).await?;
    Ok((StatusCode::CREATED, Json(article)))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    MaybeJwtAuth(auth): MaybeJwtAuth,
    Path(id): Path<String>,
) -> AppResult<Json<Article>> {
    let article = find_article_by_id(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    let can_view_unpublished = match auth {
        Some(claims) => is_admin(&state, &claims.user_id).await?,
        None => false,
    };
    if article.status.as_deref() != Some("published") && !can_view_unpublished {
        return Err(AppError::NotFound);
    }
    Ok(Json(article))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    require_admin(&state, &claims).await?;
    if delete_article_by_id(&state.pool, &id).await? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Path(id): Path<String>,
    Json(input): Json<ArticleInput>,
) -> AppResult<Json<Article>> {
    require_admin(&state, &claims).await?;
    if input.id.as_deref().is_some_and(|body_id| body_id != id) {
        return Err(AppError::BadRequest(
            "article id does not match request path".into(),
        ));
    }
    validate_article_input(&input)?;
    update_article(&state.pool, &id, input)
        .await
        .map(Json)
        .map_err(map_row_not_found)
}

pub async fn update_status(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Path(id): Path<String>,
    Json(input): Json<ArticleStatusInput>,
) -> AppResult<Json<Article>> {
    require_admin(&state, &claims).await?;
    if !matches!(input.status.as_str(), "draft" | "published" | "archived") {
        return Err(AppError::BadRequest("invalid article status".into()));
    }
    update_article_status(&state.pool, &id, input)
        .await
        .map(Json)
        .map_err(map_row_not_found)
}

fn validate_article_input(input: &ArticleInput) -> AppResult<()> {
    if input
        .title
        .as_deref()
        .is_none_or(|title| title.trim().is_empty())
    {
        return Err(AppError::BadRequest("article title is required".into()));
    }
    Ok(())
}

fn map_row_not_found(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Sqlx(other),
    }
}
