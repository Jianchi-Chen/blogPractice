//! Comment Repository - 评论数据访问层

use chrono::Utc;
use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::comment::{Comment, CommentWithLike};

#[derive(Deserialize, Debug, Clone)]
pub struct CommentIncome {
    pub article_id: String,
    pub content: String,
    pub parent_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LikeCommentPayload {
    pub comment_id: String,
}

/// 获取文章的评论列表
pub async fn fetch_comments_by_article_id(
    pool: &SqlitePool,
    article_id: &str,
    uid: &str,
) -> Result<Vec<CommentWithLike>, sqlx::Error> {
    sqlx::query_as::<_, CommentWithLike>(
        r#"
        SELECT  
            c.comment_id,
            c.article_id,
            c.user,
            c.content,
            c.created_at,
            c.parent_id,
            c.like_count,
            CASE WHEN cl.user_id IS NOT NULL THEN 1 ELSE 0 END AS liked_by_me
        FROM comments AS c
        LEFT JOIN comment_likes AS cl
            ON cl.comment_id = c.comment_id
            AND cl.user_id = ?
        WHERE c.article_id = ?
        ORDER BY c.created_at DESC
        "#,
    )
    .bind(uid)
    .bind(article_id)
    .fetch_all(pool)
    .await
}

/// 发布评论
pub async fn post_comment_by_article_id(
    pool: &SqlitePool,
    new: CommentIncome,
    username: &str,
) -> Result<Comment, sqlx::Error> {
    let c_id = Uuid::now_v7().to_string();
    let create_at = Utc::now().to_rfc3339();

    sqlx::query_as::<_, Comment>(
        r#"
        INSERT INTO comments 
        (comment_id, article_id, user, content, created_at, parent_id) 
        VALUES (?,?,?,?,?,?) 
        RETURNING *
        "#,
    )
    .bind(c_id)
    .bind(new.article_id)
    .bind(username)
    .bind(new.content)
    .bind(create_at)
    .bind(new.parent_id)
    .fetch_one(pool)
    .await
}

/// 删除评论
pub async fn delete_comment_by_comment_id(
    pool: &SqlitePool,
    comment_id: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(r#"DELETE FROM comments WHERE comment_id = ?"#)
        .bind(comment_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// 点赞/取消点赞评论
pub async fn like_comment_db(
    pool: &SqlitePool,
    payload: LikeCommentPayload,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    let comment_id = &payload.comment_id;
    let mut transaction = pool.begin().await?;
    let deleted = sqlx::query(r#"DELETE FROM comment_likes WHERE comment_id = ? AND user_id = ?"#)
        .bind(comment_id)
        .bind(user_id)
        .execute(&mut *transaction)
        .await?
        .rows_affected()
        == 1;

    let result = if deleted {
        "unliked"
    } else {
        let created_at = Utc::now().to_rfc3339();
        let inserted = sqlx::query(
            r#"
            INSERT INTO comment_likes (comment_id, user_id, created_at, article_id)
            SELECT comment_id, ?, ?, article_id
            FROM comments
            WHERE comment_id = ?
            "#,
        )
        .bind(user_id)
        .bind(created_at)
        .bind(comment_id)
        .execute(&mut *transaction)
        .await?;

        if inserted.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
        "liked"
    };

    let updated = sqlx::query(
        r#"
        UPDATE comments
        SET like_count = (
            SELECT COUNT(*) FROM comment_likes WHERE comment_id = ?
        )
        WHERE comment_id = ?
        "#,
    )
    .bind(comment_id)
    .bind(comment_id)
    .execute(&mut *transaction)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    transaction.commit().await?;
    Ok(result.to_string())
}
