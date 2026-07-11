use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    auth::{Claims, JwtAuth, MaybeJwtAuth, is_admin, require_admin},
    db::AppState,
    error::{AppError, AppResult},
    models::comment::{
        Comment, CommentLikeState, CommentWithLike, CreateCommentInput, SetCommentLikeInput,
    },
    repositories::{
        article::find_article_by_id,
        comment::{create_comment, delete_comment_by_id, list_comments, set_comment_like},
        user::{find_user_by_id, get_identity},
    },
};

#[derive(Serialize)]
pub struct CommentListResponse {
    pub comments: Vec<CommentWithLike>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Path(article_id): Path<String>,
    MaybeJwtAuth(auth): MaybeJwtAuth,
) -> AppResult<Json<CommentListResponse>> {
    let user_id = auth
        .as_ref()
        .map(|claims| claims.user_id.as_str())
        .unwrap_or_default();
    ensure_article_visible(&state, &article_id, user_id).await?;
    let comments = list_comments(&state.pool, &article_id, user_id).await?;
    Ok(Json(CommentListResponse { comments }))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Path(article_id): Path<String>,
    Json(input): Json<CreateCommentInput>,
) -> AppResult<(StatusCode, Json<Comment>)> {
    let comment = create_for_article(&state, &claims, &article_id, input).await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

pub(crate) async fn create_for_article(
    state: &AppState,
    claims: &Claims,
    article_id: &str,
    input: CreateCommentInput,
) -> AppResult<Comment> {
    ensure_article_visible(state, article_id, &claims.user_id).await?;
    validate_comment(&input)?;
    let user = find_user_by_id(&state.pool, &claims.user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("user does not exist".into()))?;
    if user.identity == "visitor" {
        return Err(AppError::Forbidden);
    }
    create_comment(&state.pool, article_id, input, &user.username)
        .await
        .map_err(AppError::Sqlx)
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Path(comment_id): Path<String>,
) -> AppResult<StatusCode> {
    delete_by_id(&state, &claims, &comment_id).await
}

pub(crate) async fn delete_by_id(
    state: &AppState,
    claims: &Claims,
    comment_id: &str,
) -> AppResult<StatusCode> {
    require_admin(state, claims).await?;
    if delete_comment_by_id(&state.pool, comment_id).await? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_like(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Path(comment_id): Path<String>,
    Json(input): Json<SetCommentLikeInput>,
) -> AppResult<Json<CommentLikeState>> {
    ensure_can_interact(&state, &claims.user_id).await?;
    set_comment_like(&state.pool, &comment_id, &claims.user_id, input.liked)
        .await
        .map(Json)
        .map_err(map_row_not_found)
}

async fn ensure_article_visible(
    state: &AppState,
    article_id: &str,
    user_id: &str,
) -> AppResult<()> {
    let article = find_article_by_id(&state.pool, article_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if article.status.as_deref() != Some("published")
        && (user_id.is_empty() || !is_admin(state, user_id).await?)
    {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub(crate) async fn ensure_can_interact(state: &AppState, user_id: &str) -> AppResult<()> {
    let identity = get_identity(&state.pool, user_id)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => AppError::Unauthorized("user does not exist".into()),
            other => AppError::Sqlx(other),
        })?;
    if matches!(identity.as_str(), "admin" | "user") {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn validate_comment(input: &CreateCommentInput) -> AppResult<()> {
    if input.content.trim().is_empty() {
        return Err(AppError::BadRequest("comment content is required".into()));
    }
    Ok(())
}

pub(crate) fn map_row_not_found(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Sqlx(other),
    }
}
