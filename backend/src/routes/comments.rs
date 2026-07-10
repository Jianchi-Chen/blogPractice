use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{JwtAuth, MaybeJwtAuth},
    db::AppState,
    error::{AppError, AppResult},
    models::{
        article::find_article_by_id,
        comment::{
            Comment, CommentWithLike, delete_comment_by_comment_id, fetch_comments_by_article_id,
            like_comment_db, post_comment_by_article_id,
        },
        user::find_user_by_id,
    },
    routes::auth::get_ident_by_id,
};

#[derive(Serialize, Deserialize)]
pub struct CommentsResponse {
    pub comments: Vec<CommentWithLike>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentIncome {
    pub article_id: String,
    pub user_id: Option<String>,
    pub content: String,
    pub parent_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeleteCommentParams {
    pub comment_id: String,
    // pub article_id: String,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[warn(non_snake_case)]
pub struct LikeCommentPayload {
    pub comment_id: String,
}

#[derive(Serialize)]
pub struct CommentsLikeResponse {
    pub comment_id: String,
    pub like_or_unlike: String,
}

/// 获取评论
pub async fn handle_get_comments(
    State(state): State<Arc<AppState>>,
    Path(arti_id): Path<String>,
    MaybeJwtAuth(maybe_auth): MaybeJwtAuth,
) -> AppResult<Json<CommentsResponse>> {
    tracing::info!("Fetching comments for article ID: {:?}", arti_id);

    let arti_id = match find_article_by_id(&state.pool, &arti_id).await? {
        Some(v) => v.id,
        None => return Err(AppError::BadRequest("未找到文章".into())),
    };

    // 获取用户ID（如果有token）
    let user_id = maybe_auth.map(|claims| claims.user_id).unwrap_or_default();

    let res = fetch_comments_by_article_id(&state.pool, &arti_id, &user_id).await?;

    Ok(Json(CommentsResponse { comments: res }))
}

/// 发表评论
// 有个坑，jwt如果放在后面，axum提取器可能不会识别从而报错
pub async fn handle_post_comment(
    State(state): State<Arc<AppState>>,
    JwtAuth(auth): JwtAuth,
    Json(payload): Json<CommentIncome>,
) -> AppResult<Json<Comment>> {
    let user = find_user_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户不存在".into()))?;
    if user.identity == "visitor" {
        tracing::info!("游客身份,禁止评论,当前用户身份: {}", user.identity);
        return Err(AppError::Unauthorized("未登录".into()));
    }
    let username = user.username;

    let res = if (find_article_by_id(&state.pool, &payload.article_id).await?).is_some() {
        post_comment_by_article_id(&state.pool, payload, &username).await?
    } else {
        return Err(AppError::BadRequest("未找到文章".into()));
    };
    tracing::info!("用户 {} 发表评论 {:?} 成功", username, res.content.clone());
    Ok(Json(res))
}

/// 删除评论
pub async fn handle_delete_comment(
    State(state): State<Arc<AppState>>,
    JwtAuth(auth): JwtAuth,
    Path(comment_id): Path<String>,
) -> AppResult<Json<DeleteCommentParams>> {
    if get_ident_by_id(&state.pool, &auth.user_id).await? != "admin" {
        return Err(AppError::Unauthorized("权限不足".into()));
    };

    if delete_comment_by_comment_id(&state.pool, &comment_id).await? != "done" {
        return Err(AppError::BadRequest("删除失败".into()));
    }

    let res = DeleteCommentParams {
        comment_id,
        message: Some("deletion operation completed".to_string()),
    };

    Ok(Json(res))
}

// 点赞评论
pub async fn like_comment(
    State(state): State<Arc<AppState>>,
    JwtAuth(auth): JwtAuth,
    Json(payload): Json<LikeCommentPayload>,
) -> AppResult<Json<CommentsLikeResponse>> {
    tracing::info!(
        "trying to like comment with ID: {:?} by user ID: {:?}",
        &payload.comment_id,
        auth.user_id
    );
    // 权限检查
    if (find_user_by_id(&state.pool, auth.user_id.clone()).await?).is_some() {
    } else {
        return Err(AppError::BadRequest("用户不存在，无法点赞".into()));
    }
    let ident = get_ident_by_id(&state.pool, &auth.user_id).await?;
    if ident != "admin" && ident != "user" {
        return Err(AppError::Unauthorized("权限不足，无法点赞".into()));
    }

    let res = like_comment_db(&state.pool, payload.clone(), &auth.user_id).await?;

    tracing::info!("Like or unlike comment successfully: {:?}", res);

    Ok(Json(CommentsLikeResponse {
        comment_id: payload.comment_id,
        like_or_unlike: res,
    }))
}

// todo 也许这是不需要的接口
// 获取评论点赞情况
// pub async fn get_comments_like(
//     State(state): State<Arc<AppState>>,
//     Query(article_id): Query<CommentsQuery>,
// ) -> AppResult<Json<Vec<LikeCommentPayload>>> {
//     tracing::info!(
//         "trying to fetch comments like for article ID: {:?}",
//         article_id
//     );

//     let res = get_comments_liked_db(&state.pool, &article_id.article_id).await?;

//     tracing::info!("Fetched comments like successfully");

//     Ok(Json(res))
// }
