use chrono::Utc;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::{
    error::AppError,
    routes::articles::{GetArticlesParams, NewArticle, NewStatus},
};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ArticleModel {
    pub id: String, // 如果数据库的字段可为NULL，rust的字段也必须要可为None
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub created_at: Option<String>,
    pub update_at: Option<String>,
    pub status: Option<String>,
    pub views: Option<i32>,
    pub tags: Option<String>,
}

impl From<ArticleModel> for NewArticle {
    fn from(v: ArticleModel) -> Self {
        Self {
            id: Some(v.id),
            title: v.title,
            content: v.content,
            summary: v.summary,
            status: v.status,
            tags: v.tags,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct PubArticles {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub created_at: String,
    pub status: String,
    pub views: i32,
    pub tags: String,
}

/// 获取文章列表
pub async fn get_articles(
    pool: &SqlitePool,
    params: GetArticlesParams,
    include_unpublished: bool,
) -> Result<Vec<PubArticles>, sqlx::Error> {
    let rows = if include_unpublished {
        sqlx::query_as::<_, PubArticles>(
            // r#"..."# Rust原始字符串(raw string)语法，被包裹内容不会被转义
            r#"
            SELECT id, title, summary, created_at, status, views, tags FROM articles
            "#,
        )
        .fetch_all(pool) //执行语句
        .await
        .inspect_err(|e| {
            eprintln!("DB error: {:?}", e);
        })?
    } else {
        sqlx::query_as::<_, PubArticles>(
            // r#"..."# Rust原始字符串(raw string)语法，被包裹内容不会被转义
            r#"
            SELECT id, title, summary, created_at, status, views, tags FROM articles
            WHERE status = ?
            "#,
        )
        .bind("published") // 需要uuid的feature
        .fetch_all(pool) //执行语句
        .await
        .inspect_err(|e| {
            eprintln!("DB error: {:?}", e);
        })?
    };

    // 再进行一次关键词搜索
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
    let create_at = Utc::now().to_rfc3339();
    let status = "draft".to_string();

    sqlx::query(
        // r#"..."# Rust原始字符串(raw string)语法，被包裹内容不会被转义
        r#"
        INSERT INTO articles (id, title, content, summary, created_at, status, tags)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id) // 需要uuid的feature
    .bind(&new.title)
    .bind(&new.content)
    .bind(&new.summary)
    .bind(create_at)
    .bind(status)
    .bind(&new.tags) // Option类型数据库会自己处理空值逻辑，没有的话就存NULL
    .execute(pool) //执行语句，并 等待一行结果
    .await
    .map_err(|e| {
        eprintln!("DB error: {:?}", e);
        e
    })?;

    sqlx::query_as::<_, ArticleModel>(r#"SELECT * FROM articles WHERE id = ?"#)
        .bind(&id)
        .fetch_one(pool)
        .await
}

// 一般函数能给&str给&str，需要所有权时才给String
/// 查找文章
pub async fn find_article_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<ArticleModel>, sqlx::Error> {
    sqlx::query_as::<_, ArticleModel>(r#"select * from articles where id = ?"#)
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// 删除文章
pub async fn delete_article_by_id(pool: &SqlitePool, id: &str) -> Result<StatusCode, AppError> {
    // 使用query / execute 代替 query_as::<> / fetch_*
    let res = sqlx::query(r#"DELETE FROM articles where id = ?"#)
        .bind(id)
        .execute(pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

/// 修改文章
pub async fn put_article_by_id(
    pool: &SqlitePool,
    id: &str,
    new: NewArticle,
) -> Result<ArticleModel, AppError> {
    let update_at = Utc::now().to_rfc3339();

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
    .bind(new.title)
    .bind(new.content)
    .bind(new.summary)
    .bind(update_at)
    .bind(new.tags)
    .bind("draft")
    .bind(id)
    .execute(pool)
    .await?;

    Ok(
        sqlx::query_as::<_, ArticleModel>(r#"SELECT * FROM articles WHERE id = ?"#)
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                eprintln!("DB select error: {:?}", e);
                e
            })?,
    )
}

/// 更变文章状态
pub async fn patch_article_by_id(
    pool: &SqlitePool,
    id: &str,
    new: NewStatus,
) -> Result<ArticleModel, AppError> {
    let _ = sqlx::query(
        r#"
        UPDATE articles SET status = ?
        WHERE id = ?
        "#,
    )
    .bind(&new.toggle)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| {
        eprintln!("DB update error: {:?}", e);
        e
    })?;

    Ok(
        sqlx::query_as::<_, ArticleModel>(r#"SELECT * FROM articles WHERE id = ?"#)
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| {
                eprintln!("DB select error: {:?}", e);
                e
            })?,
    )
}
