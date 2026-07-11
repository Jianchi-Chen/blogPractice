use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::comment::{Comment, CommentLikeState, CommentWithLike, CreateCommentInput};

pub async fn list_comments(
    pool: &SqlitePool,
    article_id: &str,
    user_id: &str,
) -> Result<Vec<CommentWithLike>, sqlx::Error> {
    sqlx::query_as::<_, CommentWithLike>(
        r#"
        SELECT c.comment_id, c.article_id, c.user, c.content, c.created_at, c.parent_id,
               c.like_count,
               CASE WHEN cl.user_id IS NOT NULL THEN 1 ELSE 0 END AS liked_by_me
        FROM comments AS c
        LEFT JOIN comment_likes AS cl
          ON cl.comment_id = c.comment_id AND cl.user_id = ?
        WHERE c.article_id = ?
        ORDER BY c.created_at DESC
        "#,
    )
    .bind(user_id)
    .bind(article_id)
    .fetch_all(pool)
    .await
}

pub async fn create_comment(
    pool: &SqlitePool,
    article_id: &str,
    input: CreateCommentInput,
    username: &str,
) -> Result<Comment, sqlx::Error> {
    let comment_id = Uuid::now_v7().to_string();
    let created_at = Utc::now().to_rfc3339();
    sqlx::query_as::<_, Comment>(
        r#"
        INSERT INTO comments (comment_id, article_id, user, content, created_at, parent_id)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(comment_id)
    .bind(article_id)
    .bind(username)
    .bind(input.content)
    .bind(created_at)
    .bind(input.parent_id)
    .fetch_one(pool)
    .await
}

pub async fn delete_comment_by_id(pool: &SqlitePool, comment_id: &str) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM comments WHERE comment_id = ?")
        .bind(comment_id)
        .execute(pool)
        .await?
        .rows_affected())
}

pub async fn set_comment_like(
    pool: &SqlitePool,
    comment_id: &str,
    user_id: &str,
    liked: bool,
) -> Result<CommentLikeState, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    if liked {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO comment_likes (comment_id, user_id, created_at, article_id)
            SELECT comment_id, ?, ?, article_id FROM comments WHERE comment_id = ?
            "#,
        )
        .bind(user_id)
        .bind(Utc::now().to_rfc3339())
        .bind(comment_id)
        .execute(&mut *transaction)
        .await?;
    } else {
        sqlx::query("DELETE FROM comment_likes WHERE comment_id = ? AND user_id = ?")
            .bind(comment_id)
            .bind(user_id)
            .execute(&mut *transaction)
            .await?;
    }

    let updated = sqlx::query(
        r#"
        UPDATE comments
        SET like_count = (SELECT COUNT(*) FROM comment_likes WHERE comment_id = ?)
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

    let like_count =
        sqlx::query_scalar::<_, i64>("SELECT like_count FROM comments WHERE comment_id = ?")
            .bind(comment_id)
            .fetch_one(&mut *transaction)
            .await?;
    transaction.commit().await?;

    Ok(CommentLikeState {
        comment_id: comment_id.to_string(),
        liked_by_me: liked,
        like_count,
    })
}

pub async fn has_comment_like(
    pool: &SqlitePool,
    comment_id: &str,
    user_id: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM comment_likes WHERE comment_id = ? AND user_id = ?)",
    )
    .bind(comment_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}
