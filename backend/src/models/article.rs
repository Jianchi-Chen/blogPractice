use chrono::Local;
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
    let now = Local::now();
    let create_at = now.format("%Y::%m::%d").to_string();
    let status = "draft".to_string();

    // query_as 是 sqlx 的宏：它在 编译期 检查 SQL 语法，并把结果行直接 按列名映射 到你指定的结构体 User。
    // 第一个类型参数 _ 让编译器推断数据库驱动（这里是 SQLite，只有一种数据库的话可以自己推导）；第二个 User 指定目标结构体。
    let _ = sqlx::query(
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
    });

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
    let now = Local::now();
    let update_at = now.format("%Y::%m::%d").to_string();

    // 不带宏是运行期再检查
    // 使用宏是编译期检查
    let _ = sqlx::query!(
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
        new.title,
        new.content,
        new.summary,
        update_at,
        new.tags,
        "draft",
        id
    )
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
