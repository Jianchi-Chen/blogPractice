use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

// 嵌入种子数据到二进制文件中
const SEED_SUPERUSER: &str = include_str!("../seeds/0001_superuser.sql");

/// 创建新的数据库连接池
pub async fn new_pool(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // 检查数据库文件是否已存在
    let is_new_db = if db_url.starts_with("sqlite://") {
        let path = db_url.trim_start_matches("sqlite://");
        let db_path = PathBuf::from(path);
        
        let exists = db_path.exists();
        log::info!("Database file path: {}", db_path.display());
        log::info!("Database exists: {}", exists);
        
        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            log::info!("Creating database directory: {}", parent.display());
            std::fs::create_dir_all(parent)
                .map_err(|e| {
                    log::error!("Failed to create directory {}: {}", parent.display(), e);
                    sqlx::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to create database directory: {}", e)
                    ))
                })?;
            log::info!("Database directory created successfully");
        }
        
        !exists  // 返回是否是新数据库
    } else {
        false
    };

    // 添加 create_if_missing 选项到连接字符串
    let connection_url = if db_url.contains('?') {
        format!("{}&mode=rwc", db_url)
    } else {
        format!("{}?mode=rwc", db_url)
    };
    
    log::info!("Connecting to database: {}", connection_url);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&connection_url)
        .await?;
    
    // 将是否为新数据库的信息存储到连接池的元数据中（通过查询结果判断）
    if is_new_db {
        log::info!("New database detected, will run seeds after migrations");
    }
    
    Ok(pool)
}

/// 运行数据库迁移
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    log::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    log::info!("Database migrations completed successfully");
    Ok(())
}

/// 检查是否需要运行种子数据（检查用户表是否为空）
async fn should_run_seeds(pool: &SqlitePool) -> Result<bool, sqlx::Error> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    
    Ok(count.0 == 0)
}

/// 运行数据库种子数据（仅在首次创建数据库时执行）
pub async fn run_seeds(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    log::info!("Checking if seeds need to be run...");
    
    // 检查用户表是否为空
    if !should_run_seeds(pool).await? {
        log::info!("Database already has users, skipping seeds");
        return Ok(());
    }
    
    log::info!("Running seed: superuser");
    
    // 执行嵌入的种子数据
    sqlx::raw_sql(SEED_SUPERUSER)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("Failed to execute superuser seed: {}", e);
            e
        })?;
    
    log::info!("Superuser seed executed successfully");
    log::info!("Default superuser created: username=admin, password=tfF;1J(2WokG,5");
    
    Ok(())
}
