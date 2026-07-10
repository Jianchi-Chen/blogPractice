//! 数据库模块：提供连接池、迁移与全局状态。
//! 说明：将 `pool` 和 `config` 放入 `AppState`，方便在 handler 与提取器中访问。

use crate::config::Config;
use anyhow::Ok;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::env;

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
    // println!("当前工作目录: {:?}", std::env::current_dir()?);
    // panic!("{}", database_url);
    // create_dir(Path::new("test")).unwrap();
    // File::create(format!("{}", database_url))?;

    let db_file = database_url.strip_prefix("sqlite://").unwrap();
    let db_path = env::current_dir()?.join(db_file);

    // SqliteConnectOptions = 描述“怎么连到某一个数据库”的细粒度参数（文件路径、是否创建、超时、外键开关……）。
    // SqlitePoolOptions = 描述“怎么管理一堆连接”的参数（最大连接数、最小连接数、连接超时、健康检查……）。
    SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(&db_path)
                .create_if_missing(true),
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
