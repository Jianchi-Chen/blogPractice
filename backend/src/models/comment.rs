use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::routes::comments::{CommentIncome, LikeCommentPayload};

#[derive(Serialize, FromRow, Deserialize, Debug)]
pub struct Comment {
    pub comment_id: String,
    pub article_id: Option<String>,
    pub user: Option<String>,
    pub content: Option<String>,
    pub created_at: Option<String>,
    pub parent_id: Option<String>,
    pub like_count: Option<i64>,
}

#[derive(Serialize, FromRow, Deserialize, Debug)]
pub struct CommentWithLike {
    pub comment_id: String,
    pub article_id: Option<String>,
    pub user: Option<String>,
    pub content: Option<String>,
    pub created_at: Option<String>,
    pub parent_id: Option<String>,
    pub like_count: Option<i64>,
    pub liked_by_me: Option<i64>,
}

/// 获取评论
pub async fn fetch_comments_by_article_id(
    pool: &SqlitePool,
    article_id: &str,
    uid: &str,
) -> Result<Vec<CommentWithLike>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CommentWithLike>(
        r#"
        SELECT  
                c.comment_id,      -- 评论的唯一 ID
                c.article_id,      -- 所属文章 ID
                c.user,            -- 评论用户
                c.content,         -- 评论内容
                c.created_at,      -- 评论时间
                c.parent_id,       -- 父评论 ID(用于楼中楼/回复)
                c.like_count,      -- 点赞总数
                CASE WHEN cl.user_id IS NOT NULL THEN 1 ELSE 0 END AS liked_by_me
                                -- 如果当前用户在点赞表里有记录，则标记为 1,否则为 0
            FROM    comments AS c
            LEFT JOIN comment_likes AS cl
                ON cl.comment_id = c.comment_id
                AND cl.user_id     = ?   -- 只关联当前用户的点赞记录
            WHERE   c.article_id = ?       -- 只查某篇文章的评论
            ORDER BY c.created_at DESC;       -- 按时间倒序排列（最新的在前）
        "#,
    )
    .bind(uid)
    .bind(article_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        eprintln!("DB error: {:?}", e);
        e
    })?;

    Ok(rows)
}

/// 发布评论
pub async fn post_comment_by_article_id(
    pool: &SqlitePool,
    new: CommentIncome,
    username: &str,
) -> Result<Comment, sqlx::Error> {
    let c_id = Uuid::now_v7().to_string();
    let create_at = Utc::now().to_rfc3339();

    let res: Comment = sqlx::query_as::<_, Comment>(
        r#"INSERT INTO comments 
        (comment_id, article_id, user, content, created_at, parent_id) 
        VALUES (?,?,?,?,?,?) 
        RETURNING *"#,
    )
    .bind(c_id)
    .bind(new.article_id)
    .bind(username)
    .bind(new.content)
    .bind(create_at)
    .bind(new.parent_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        eprintln!("DB error: {:?}", e);
        e
    })?;

    Ok(res)
}

/// 删除评论
pub async fn delete_comment_by_comment_id(
    pool: &SqlitePool,
    comment_id: &str,
) -> Result<u64, sqlx::Error> {
    tracing::info!("Deleting comment with ID: {:?}", comment_id);
    let result = sqlx::query(r#"DELETE FROM comments WHERE comment_id = ?"#)
        .bind(comment_id)
        .execute(pool)
        .await
        .map_err(|e| {
            eprintln!("db delete error: {:?}", e);
            e
        })?;

    Ok(result.rows_affected())
}

// 点赞评论
pub async fn like_comment_db(
    pool: &SqlitePool,
    payload: LikeCommentPayload,
    user_id: &str,
) -> Result<String, sqlx::Error> {
    tracing::info!(
        "Liking comment with ID: {:?} by user ID: {:?}",
        payload.comment_id,
        user_id
    );

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
