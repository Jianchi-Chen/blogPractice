//! Temporary adapters for browser bundles deployed before the shared API.
//! Remove this module only after old Web assets no longer call these paths.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    auth::{JwtAuth, decode_token},
    db::AppState,
    error::{AppError, AppResult},
    models::{
        article::ArticleSuggestion,
        comment::{Comment, CreateCommentInput},
        user::AdminUpdateUser,
    },
    repositories::{
        article::find_suggestions,
        comment::{has_comment_like, set_comment_like},
        user::find_user_by_id,
    },
    routes::{articles, auth, comments, users},
};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/register", post(auth::register))
        .route("/api/login", post(auth::login))
        .route("/api/verify_token", post(verify_token))
        .route("/api/current_user", post(current_user))
        .route("/api/editAccount", put(update_user))
        .route("/articles", get(articles::list))
        .route("/api/article", post(articles::create))
        .route("/article/{id}", get(articles::get))
        .route(
            "/api/article/{id}",
            delete(articles::delete)
                .put(articles::update)
                .patch(articles::update_status),
        )
        .route("/comments/{id}", get(comments::list))
        .route("/api/comment", post(create_comment))
        .route("/comment/{id}", delete(delete_comment))
        .route("/api/comment/like", put(toggle_comment_like))
        .route("/suggestions/{query}", get(suggestions))
}

#[derive(Deserialize)]
struct TokenRequest {
    token: String,
}

#[derive(Serialize)]
struct TokenResponse {
    user_id: String,
    message: String,
    exp: usize,
    iat: usize,
}

async fn verify_token(
    State(state): State<Arc<AppState>>,
    Json(input): Json<TokenRequest>,
) -> AppResult<Json<TokenResponse>> {
    let claims = decode_token(&state, &input.token)
        .map_err(|_| AppError::Unauthorized("invalid token".into()))?;
    Ok(Json(TokenResponse {
        user_id: claims.user_id,
        message: claims.message,
        exp: claims.exp,
        iat: claims.iat,
    }))
}

#[derive(Serialize)]
struct LegacyCurrentUser {
    id: String,
    username: String,
    identity: String,
}

async fn current_user(
    State(state): State<Arc<AppState>>,
    Json(input): Json<TokenRequest>,
) -> AppResult<Json<LegacyCurrentUser>> {
    let claims = decode_token(&state, &input.token)
        .map_err(|_| AppError::Unauthorized("invalid token".into()))?;
    let user = find_user_by_id(&state.pool, &claims.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(LegacyCurrentUser {
        id: user.id,
        username: user.username,
        identity: user.identity,
    }))
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Json(input): Json<AdminUpdateUser>,
) -> AppResult<StatusCode> {
    users::update_account(
        &state,
        &claims,
        input.user_id,
        users::UpdateUserInput {
            username: input.username,
            password: input.password,
            identity: input.identity,
        },
    )
    .await
}

#[derive(Deserialize)]
struct LegacyCommentInput {
    article_id: String,
    content: String,
    parent_id: Option<String>,
}

async fn create_comment(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Json(input): Json<LegacyCommentInput>,
) -> AppResult<Json<Comment>> {
    comments::create_for_article(
        &state,
        &claims,
        &input.article_id,
        CreateCommentInput {
            content: input.content,
            parent_id: input.parent_id,
        },
    )
    .await
    .map(Json)
}

#[derive(Serialize)]
struct LegacyDeleteCommentResponse {
    comment_id: String,
    message: String,
}

async fn delete_comment(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Path(comment_id): Path<String>,
) -> AppResult<Json<LegacyDeleteCommentResponse>> {
    comments::delete_by_id(&state, &claims, &comment_id).await?;
    Ok(Json(LegacyDeleteCommentResponse {
        comment_id,
        message: "deletion operation completed".into(),
    }))
}

#[derive(Deserialize)]
struct LegacyLikeInput {
    comment_id: String,
}

#[derive(Serialize)]
struct LegacyLikeResponse {
    comment_id: String,
    like_or_unlike: String,
}

async fn toggle_comment_like(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Json(input): Json<LegacyLikeInput>,
) -> AppResult<Json<LegacyLikeResponse>> {
    comments::ensure_can_interact(&state, &claims.user_id).await?;
    let liked = !has_comment_like(&state.pool, &input.comment_id, &claims.user_id).await?;
    let result = set_comment_like(&state.pool, &input.comment_id, &claims.user_id, liked)
        .await
        .map_err(comments::map_row_not_found)?;
    Ok(Json(LegacyLikeResponse {
        comment_id: input.comment_id,
        like_or_unlike: if result.liked_by_me {
            "liked".into()
        } else {
            "unliked".into()
        },
    }))
}

#[derive(Serialize)]
struct LegacySuggestionResponse {
    item: Vec<ArticleSuggestion>,
}

async fn suggestions(
    State(state): State<Arc<AppState>>,
    Path(query): Path<String>,
) -> AppResult<Json<LegacySuggestionResponse>> {
    let item = if query.trim().is_empty() {
        Vec::new()
    } else {
        find_suggestions(&state.pool, query.trim_matches('"')).await?
    };
    Ok(Json(LegacySuggestionResponse { item }))
}
