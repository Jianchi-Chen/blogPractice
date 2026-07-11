use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    auth::JwtAuth,
    db::AppState,
    error::{AppError, AppResult},
    models::{article::ArticleSummary, favorite::FavoriteState},
    repositories::favorite::{add_favorite, list_favorites, remove_favorite},
};

#[derive(Debug, Serialize)]
pub struct FavoriteListResponse {
    pub articles: Vec<ArticleSummary>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
) -> AppResult<Json<FavoriteListResponse>> {
    let articles = list_favorites(&state.pool, &claims.user_id).await?;
    Ok(Json(FavoriteListResponse { articles }))
}

pub async fn add(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Path(article_id): Path<String>,
) -> AppResult<Json<FavoriteState>> {
    add_favorite(&state.pool, &claims.user_id, &article_id)
        .await
        .map_err(map_row_not_found)?;
    Ok(Json(FavoriteState {
        article_id,
        favorited: true,
    }))
}

pub async fn remove(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Path(article_id): Path<String>,
) -> AppResult<Json<FavoriteState>> {
    remove_favorite(&state.pool, &claims.user_id, &article_id).await?;
    Ok(Json(FavoriteState {
        article_id,
        favorited: false,
    }))
}

fn map_row_not_found(error: sqlx::Error) -> AppError {
    match error {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Sqlx(other),
    }
}
