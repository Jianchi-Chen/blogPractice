//! 数据库模块：提供连接池、迁移与全局状态。
//! 说明：将 `pool` 和 `config` 放入 `AppState`，方便在 handler 与提取器中访问。

use crate::config::Config;
use anyhow::Ok;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{str::FromStr, time::Duration};

// 嵌入种子数据到二进制文件中
const SEED_SUPERUSER: &str = include_str!("../../seeds/0001_superuser.sql");

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub cfg: Config,
}

impl AppState {
    pub fn new(pool: SqlitePool, cfg: Config) -> Self {
        Self { pool, cfg }
    }
}

// 连接数据库
pub async fn new_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // SqliteConnectOptions = 描述“怎么连到某一个数据库”的细粒度参数（文件路径、是否创建、超时、外键开关……）。
    // SqlitePoolOptions = 描述“怎么管理一堆连接”的参数（最大连接数、最小连接数、连接超时、健康检查……）。
    SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(
            SqliteConnectOptions::from_str(database_url)?
                .create_if_missing(true)
                .foreign_keys(true)
                .busy_timeout(Duration::from_secs(5)),
        )
        .await
}

/// 执行迁移（embed 方式，编译期打包）
/// `./migrations` 目录存放 SQL 脚本
pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;

    run_seeds(pool).await?;
    Ok(())
}

/// 检查是否需要运行种子数据（检查用户表是否为空）
async fn should_run_seeds(pool: &SqlitePool) -> anyhow::Result<bool> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    Ok(count.0 == 0)
}

/// 执行种子数据（仅在数据库为空时执行）
async fn run_seeds(pool: &SqlitePool) -> anyhow::Result<()> {
    // 检查用户表是否为空
    if !should_run_seeds(pool).await? {
        tracing::info!("Database already has users, skipping seeds");
        return Ok(());
    }

    tracing::info!("Running seed: superuser");

    // 执行嵌入的种子数据
    sqlx::raw_sql(SEED_SUPERUSER).execute(pool).await?;

    tracing::info!("Superuser seed executed successfully");
    tracing::info!("Default superuser created: username=admin");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::{article::delete_article_by_id, comment::set_comment_like};
    use std::str::FromStr;

    async fn migrated_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::from_str("sqlite::memory:")
                    .unwrap()
                    .foreign_keys(true),
            )
            .await
            .unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    async fn insert_article(pool: &SqlitePool, id: &str) {
        sqlx::query("INSERT INTO articles (id) VALUES (?)")
            .bind(id)
            .execute(pool)
            .await
            .unwrap();
    }

    async fn insert_user(pool: &SqlitePool, id: &str, username: &str) {
        sqlx::query(
            "INSERT INTO users (id, username, password, identity) VALUES (?, ?, 'hash', 'user')",
        )
        .bind(id)
        .bind(username)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn insert_comment(
        pool: &SqlitePool,
        id: &str,
        article_id: &str,
        parent_id: Option<&str>,
    ) {
        sqlx::query(
            r#"
            INSERT INTO comments (comment_id, article_id, content, created_at, parent_id)
            VALUES (?, ?, 'content', '2026-01-01T00:00:00Z', ?)
            "#,
        )
        .bind(id)
        .bind(article_id)
        .bind(parent_id)
        .execute(pool)
        .await
        .unwrap();
    }

    async fn count(pool: &SqlitePool, table: &str) -> i64 {
        let query = format!("SELECT COUNT(*) FROM {table}");
        sqlx::query_scalar(&query).fetch_one(pool).await.unwrap()
    }

    #[tokio::test]
    async fn duplicate_usernames_are_rejected() {
        let pool = migrated_pool().await;
        insert_user(&pool, "user-1", "duplicate").await;

        let error = sqlx::query(
            "INSERT INTO users (id, username, password, identity) VALUES ('user-2', 'duplicate', 'hash', 'user')",
        )
        .execute(&pool)
        .await
        .unwrap_err();

        assert!(
            error
                .as_database_error()
                .is_some_and(|database_error| database_error.is_unique_violation())
        );
    }

    #[tokio::test]
    async fn deleting_parent_cascades_replies_and_likes() {
        let pool = migrated_pool().await;
        insert_article(&pool, "article-1").await;
        insert_user(&pool, "user-1", "user-1").await;
        insert_comment(&pool, "parent", "article-1", None).await;
        insert_comment(&pool, "reply", "article-1", Some("parent")).await;

        for comment_id in ["parent", "reply"] {
            set_comment_like(&pool, comment_id, "user-1", true)
                .await
                .unwrap();
        }

        let rows_affected = sqlx::query("DELETE FROM comments WHERE comment_id = 'parent'")
            .execute(&pool)
            .await
            .unwrap()
            .rows_affected();

        assert_eq!(rows_affected, 1);
        assert_eq!(count(&pool, "comments").await, 0);
        assert_eq!(count(&pool, "comment_likes").await, 0);
    }

    #[tokio::test]
    async fn deleting_article_cascades_comments_and_likes() {
        let pool = migrated_pool().await;
        insert_article(&pool, "article-1").await;
        insert_user(&pool, "user-1", "user-1").await;
        insert_comment(&pool, "comment-1", "article-1", None).await;
        set_comment_like(&pool, "comment-1", "user-1", true)
            .await
            .unwrap();

        delete_article_by_id(&pool, "article-1").await.unwrap();

        assert_eq!(count(&pool, "comments").await, 0);
        assert_eq!(count(&pool, "comment_likes").await, 0);
    }

    #[tokio::test]
    async fn like_state_keeps_record_and_count_in_sync() {
        let pool = migrated_pool().await;
        insert_article(&pool, "article-1").await;
        insert_user(&pool, "user-1", "user-1").await;
        insert_comment(&pool, "comment-1", "article-1", None).await;

        for (liked, expected_count) in [(true, 1_i64), (false, 0_i64)] {
            let result = set_comment_like(&pool, "comment-1", "user-1", liked)
                .await
                .unwrap();
            let like_count: i64 = sqlx::query_scalar(
                "SELECT like_count FROM comments WHERE comment_id = 'comment-1'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(result.liked_by_me, liked);
            assert_eq!(result.like_count, expected_count);
            assert_eq!(count(&pool, "comment_likes").await, expected_count);
            assert_eq!(like_count, expected_count);

            if liked {
                let created_at: String = sqlx::query_scalar(
                    "SELECT created_at FROM comment_likes WHERE comment_id = 'comment-1'",
                )
                .fetch_one(&pool)
                .await
                .unwrap();
                assert!(chrono::DateTime::parse_from_rfc3339(&created_at).is_ok());
            }
        }
    }
}
