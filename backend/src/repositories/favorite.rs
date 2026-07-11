use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::article::ArticleSummary;

pub async fn list_favorites(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Vec<ArticleSummary>, sqlx::Error> {
    sqlx::query_as::<_, ArticleSummary>(
        r#"
        SELECT a.id, COALESCE(a.title, '') AS title, COALESCE(a.summary, '') AS summary,
               COALESCE(a.created_at, '') AS created_at,
               COALESCE(a.status, 'draft') AS status, COALESCE(a.views, 0) AS views,
               COALESCE(a.tags, '') AS tags
        FROM favorites AS f
        JOIN articles AS a ON a.id = f.article_id
        WHERE f.user_id = ? AND a.status = 'published'
        ORDER BY f.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn add_favorite(
    pool: &SqlitePool,
    user_id: &str,
    article_id: &str,
) -> Result<(), sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM articles WHERE id = ? AND status = 'published')",
    )
    .bind(article_id)
    .fetch_one(pool)
    .await?;
    if !exists {
        return Err(sqlx::Error::RowNotFound);
    }
    sqlx::query(
        "INSERT OR IGNORE INTO favorites (user_id, article_id, created_at) VALUES (?, ?, ?)",
    )
    .bind(user_id)
    .bind(article_id)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_favorite(
    pool: &SqlitePool,
    user_id: &str,
    article_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM favorites WHERE user_id = ? AND article_id = ?")
        .bind(user_id)
        .bind(article_id)
        .execute(pool)
        .await?;
    Ok(())
}
