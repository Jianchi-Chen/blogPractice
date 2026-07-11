use axum::{
    Json,
    extract::{Query, State},
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    db::AppState,
    error::AppResult,
    models::article::{ArticleSuggestion, SearchQuery},
    repositories::article::find_suggestions,
};

#[derive(Debug, Serialize)]
pub struct SuggestionListResponse {
    pub items: Vec<ArticleSuggestion>,
}

pub async fn suggestions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<SuggestionListResponse>> {
    let items = if query.query.trim().is_empty() {
        Vec::new()
    } else {
        find_suggestions(&state.pool, &query.query).await?
    };
    Ok(Json(SuggestionListResponse { items }))
}
