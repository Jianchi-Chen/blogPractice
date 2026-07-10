//! 评论相关命令

use crate::auth::{authenticated_user, decode_token, require_admin};
use crate::config::Config;
use crate::models::comment::{Comment, CommentWithLike};
use crate::repositories::comment::*;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Serialize)]
pub struct CommentsResponse {
    comments: Vec<CommentWithLike>,
}

#[derive(serde::Serialize)]
pub struct CommentsLikeResponse {
    comment_id: String,
    like_or_unlike: String,
}

/// 获取文章评论
#[tauri::command]
pub async fn get_comments(
    article_id: String,
    token: Option<String>,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<CommentsResponse, String> {
    log::info!("attempt to get comments for article_id: {}", article_id);
    // 获取用户ID（如果有token）
    let user_id = if let Some(token) = token {
        decode_token(&config, &token)
            .ok()
            .map(|claims| claims.user_id)
            .unwrap_or_default()
    } else {
        String::new()
    };

    let comments = fetch_comments_by_article_id(pool.inner(), &article_id, &user_id)
        .await
        .map_err(|e| format!("Failed to fetch comments: {}", e))?;

    log::info!(
        "fetched {} comments for article_id: {}",
        comments.len(),
        article_id
    );

    Ok(CommentsResponse { comments })
}

/// 发布评论
#[tauri::command]
pub async fn post_comment(
    token: String,
    comment_data: CommentIncome,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<Comment, String> {
    let user = authenticated_user(&config, pool.inner(), &token).await?;
    if user.identity == "visitor" {
        return Err("当前用户无权发表评论".to_string());
    }

    let comment = post_comment_by_article_id(pool.inner(), comment_data, &user.username)
        .await
        .map_err(|e| format!("Failed to post comment: {}", e))?;

    Ok(comment)
}

/// 删除评论
#[tauri::command]
pub async fn delete_comment(
    token: String,
    comment_id: String,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<(), String> {
    require_admin(&config, pool.inner(), &token).await?;

    let rows_affected = delete_comment_by_comment_id(pool.inner(), &comment_id)
        .await
        .map_err(|e| format!("Failed to delete comment: {}", e))?;
    if rows_affected == 0 {
        return Err("Comment not found".to_string());
    }

    Ok(())
}

/// 点赞/取消点赞评论
#[tauri::command]
pub async fn like_comment(
    token: String,
    payload: LikeCommentPayload,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<CommentsLikeResponse, String> {
    // 验证 token 并获取用户ID
    let claims = decode_token(&config, &token).map_err(|e| format!("Invalid token: {}", e))?;

    let comment_id = payload.comment_id.clone();
    let result = like_comment_db(pool.inner(), payload, &claims.user_id)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => "Comment not found".to_string(),
            error => format!("Failed to like comment: {}", error),
        })?;

    Ok(CommentsLikeResponse {
        comment_id,
        like_or_unlike: result,
    })
}
