//! Tauri 应用库入口

pub mod auth;
pub mod commands;
pub mod config;
pub mod db;
pub mod models;
pub mod repositories;
pub mod tray;

use crate::tray::load_system_tray;
use config::Config;
use db::{new_pool, run_migrations, run_seeds};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            log::info!("=== Application Initialization Started ===");

            // 加载系统托盘
            load_system_tray(app)?;
            log::info!("System tray loaded successfully");

            // 加载配置
            let config = match Config::load(app.handle()) {
                Ok(cfg) => {
                    log::info!("Configuration loaded successfully");
                    cfg
                }
                Err(e) => {
                    let error_msg = format!("Failed to load configuration: {}", e);
                    eprintln!("{}", error_msg);
                    log::error!("{}", error_msg);
                    return Err(Box::new(std::io::Error::other(error_msg)));
                }
            };

            log::info!(
                "Application config: host={}:{}, jwt_ttl={}s",
                config.host,
                config.port,
                config.jwt_ttl
            );

            // 初始化数据库
            let database_url = config.get_database_path(app.handle());
            log::info!("Database URL: {}", database_url);

            let pool = match tauri::async_runtime::block_on(async {
                log::info!("Creating database connection pool...");
                let pool = new_pool(&database_url).await.map_err(|e| {
                    let msg = format!("Failed to create database pool: {}", e);
                    log::error!("{}", msg);
                    msg
                })?;

                log::info!("Running database migrations...");
                run_migrations(&pool).await.map_err(|e| {
                    let msg = format!("Failed to run migrations: {}", e);
                    log::error!("{}", msg);
                    msg
                })?;

                // 运行种子数据（仅在数据库为空时）
                log::info!("Checking and running seeds if needed...");
                run_seeds(&pool).await.map_err(|e| {
                    let msg = format!("Failed to run seeds: {}", e);
                    log::error!("{}", msg);
                    msg
                })?;

                log::info!("Database initialization completed successfully");
                Ok::<_, String>(pool)
            }) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Database initialization failed: {}", e);
                    log::error!("Database initialization failed: {}", e);
                    return Err(Box::new(std::io::Error::other(format!(
                        "Database error: {}",
                        e
                    ))));
                }
            };

            log::info!("Storing application state...");
            // 将配置和连接池存储到状态中
            app.manage(config);
            app.manage(pool);

            log::info!("=== Application Initialization Completed Successfully ===");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // HTTP 请求代理
            commands::http_request,
            // 认证
            commands::login,
            commands::register,
            commands::verify_token,
            commands::get_current_user,
            // 用户管理
            commands::get_users,
            commands::delete_user,
            commands::edit_account,
            // 文章
            commands::get_articles,
            commands::get_article_by_id,
            commands::create_article,
            commands::update_article,
            commands::delete_article,
            commands::toggle_article_status,
            // 评论
            commands::get_comments,
            commands::post_comment,
            commands::delete_comment,
            commands::like_comment,
            // 搜索
            commands::get_suggestions,
            // 用户操作
            commands::save_avatar,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
