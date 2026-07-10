//! Article Repository - 文章数据访问层

use chrono::Local;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::article::{ArticleModel, PubArticles};

#[derive(Deserialize, Debug)]
pub struct GetArticlesParams {
    pub condition: Option<String>,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct NewArticle {
    pub id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
    pub tags: Option<String>,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct NewStatus {
    pub toggle: String,
}

/// 获取文章列表
pub async fn get_articles(
    pool: &SqlitePool,
    params: GetArticlesParams,
    include_unpublished: bool,
) -> Result<Vec<PubArticles>, sqlx::Error> {
    let rows = if include_unpublished {
        sqlx::query_as::<_, PubArticles>(
            r#"
            SELECT id, title, summary, created_at, status, views, tags FROM articles
            "#,
        )
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, PubArticles>(
            r#"
            SELECT id, title, summary, created_at, status, views, tags FROM articles
            WHERE status = ?
            "#,
        )
        .bind("published")
        .fetch_all(pool)
        .await?
    };

    // 关键词搜索
    if let Some(keyword) = params.condition {
        let filtered_rows: Vec<PubArticles> = rows
            .into_iter()
            .filter(|c| c.title.contains(&keyword))
            .collect();
        return Ok(filtered_rows);
    }

    Ok(rows)
}

/// 新增文章
pub async fn post_article(
    pool: &SqlitePool,
    new: &NewArticle,
) -> Result<ArticleModel, sqlx::Error> {
    let id = Uuid::now_v7().to_string();
    let now = Local::now();
    let create_at = now.format("%Y::%m::%d").to_string();
    let status = "draft".to_string();

    sqlx::query(
        r#"
        INSERT INTO articles (id, title, content, summary, created_at, status, tags)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&new.title)
    .bind(&new.content)
    .bind(&new.summary)
    .bind(create_at)
    .bind(status)
    .bind(&new.tags)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, ArticleModel>(r#"SELECT *, '' as message FROM articles WHERE id = ?"#)
        .bind(&id)
        .fetch_one(pool)
        .await
}

/// 查找文章
pub async fn find_article_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<ArticleModel>, sqlx::Error> {
    sqlx::query_as::<_, ArticleModel>(r#"SELECT *, '' as message FROM articles WHERE id = ?"#)
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// 删除文章
pub async fn delete_article_by_id(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(r#"DELETE FROM articles WHERE id = ?"#)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// 修改文章
pub async fn put_article_by_id(
    pool: &SqlitePool,
    id: &str,
    new: NewArticle,
) -> Result<ArticleModel, sqlx::Error> {
    let now = Local::now();
    let update_at = now.format("%Y::%m::%d").to_string();

    sqlx::query(
        r#"
        UPDATE articles
        SET title = ?,
            content = ?,
            summary = ?,
            update_at = ?,
            tags = ?,
            status = ?
        WHERE id = ?
        "#,
    )
    .bind(&new.title)
    .bind(&new.content)
    .bind(&new.summary)
    .bind(&update_at)
    .bind(&new.tags)
    .bind("draft")
    .bind(id)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, ArticleModel>(r#"SELECT *, '' as message FROM articles WHERE id = ?"#)
        .bind(id)
        .fetch_one(pool)
        .await
}

/// 更变文章状态
pub async fn patch_article_by_id(
    pool: &SqlitePool,
    id: &str,
    new: NewStatus,
) -> Result<ArticleModel, sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE articles SET status = ?
        WHERE id = ?
        "#,
    )
    .bind(&new.toggle)
    .bind(id)
    .execute(pool)
    .await?;

    let mut article =
        sqlx::query_as::<_, ArticleModel>(r#"SELECT *, '' as message FROM articles WHERE id = ?"#)
            .bind(id)
            .fetch_one(pool)
            .await?;

    article.message = "done".to_string();
    Ok(article)
}
