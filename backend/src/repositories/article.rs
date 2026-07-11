use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::article::{
    Article, ArticleInput, ArticleListQuery, ArticleStatusInput, ArticleSuggestion, ArticleSummary,
};

const ARTICLE_SUMMARY_COLUMNS: &str = "id, COALESCE(title, '') AS title, COALESCE(summary, '') AS summary, \
     COALESCE(created_at, '') AS created_at, COALESCE(status, 'draft') AS status, \
     COALESCE(views, 0) AS views, COALESCE(tags, '') AS tags";

pub async fn list_articles(
    pool: &SqlitePool,
    params: ArticleListQuery,
    include_unpublished: bool,
) -> Result<Vec<ArticleSummary>, sqlx::Error> {
    let pattern = params
        .query
        .map(|query| format!("%{}%", query.trim()))
        .filter(|query| query != "%%");

    match (include_unpublished, pattern) {
        (true, None) => {
            sqlx::query_as::<_, ArticleSummary>(&format!(
                "SELECT {ARTICLE_SUMMARY_COLUMNS} FROM articles ORDER BY created_at DESC"
            ))
            .fetch_all(pool)
            .await
        }
        (false, None) => {
            sqlx::query_as::<_, ArticleSummary>(&format!(
                "SELECT {ARTICLE_SUMMARY_COLUMNS} FROM articles \
                 WHERE status = 'published' ORDER BY created_at DESC"
            ))
            .fetch_all(pool)
            .await
        }
        (true, Some(pattern)) => {
            sqlx::query_as::<_, ArticleSummary>(&format!(
                "SELECT {ARTICLE_SUMMARY_COLUMNS} FROM articles \
                 WHERE title LIKE ? COLLATE NOCASE \
                    OR summary LIKE ? COLLATE NOCASE \
                    OR tags LIKE ? COLLATE NOCASE \
                 ORDER BY created_at DESC"
            ))
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(pool)
            .await
        }
        (false, Some(pattern)) => {
            sqlx::query_as::<_, ArticleSummary>(&format!(
                "SELECT {ARTICLE_SUMMARY_COLUMNS} FROM articles \
                 WHERE status = 'published' \
                   AND (title LIKE ? COLLATE NOCASE \
                     OR summary LIKE ? COLLATE NOCASE \
                     OR tags LIKE ? COLLATE NOCASE) \
                 ORDER BY created_at DESC"
            ))
            .bind(&pattern)
            .bind(&pattern)
            .bind(&pattern)
            .fetch_all(pool)
            .await
        }
    }
}

pub async fn create_article(
    pool: &SqlitePool,
    input: &ArticleInput,
) -> Result<Article, sqlx::Error> {
    let id = Uuid::now_v7().to_string();
    let created_at = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO articles (id, title, content, summary, created_at, status, tags)
        VALUES (?, ?, ?, ?, ?, 'draft', ?)
        "#,
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&input.content)
    .bind(&input.summary)
    .bind(created_at)
    .bind(&input.tags)
    .execute(pool)
    .await?;

    find_article_by_id(pool, &id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn find_article_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<Article>, sqlx::Error> {
    sqlx::query_as::<_, Article>("SELECT * FROM articles WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn delete_article_by_id(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM articles WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected())
}

pub async fn update_article(
    pool: &SqlitePool,
    id: &str,
    input: ArticleInput,
) -> Result<Article, sqlx::Error> {
    let updated_at = Utc::now().to_rfc3339();
    let result = sqlx::query(
        r#"
        UPDATE articles
        SET title = ?, content = ?, summary = ?, update_at = ?, tags = ?, status = 'draft'
        WHERE id = ?
        "#,
    )
    .bind(input.title)
    .bind(input.content)
    .bind(input.summary)
    .bind(updated_at)
    .bind(input.tags)
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    find_article_by_id(pool, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn update_article_status(
    pool: &SqlitePool,
    id: &str,
    input: ArticleStatusInput,
) -> Result<Article, sqlx::Error> {
    let result = sqlx::query("UPDATE articles SET status = ? WHERE id = ?")
        .bind(input.status)
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }
    find_article_by_id(pool, id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn find_suggestions(
    pool: &SqlitePool,
    query: &str,
) -> Result<Vec<ArticleSuggestion>, sqlx::Error> {
    let pattern = format!("%{}%", query.trim());
    sqlx::query_as::<_, ArticleSuggestion>(
        r#"
        SELECT id, COALESCE(title, '') AS title
        FROM articles
        WHERE status = 'published' AND title LIKE ? COLLATE NOCASE
        ORDER BY title
        LIMIT 10
        "#,
    )
    .bind(pattern)
    .fetch_all(pool)
    .await
}
