//! Commands 模块 - 所有 Tauri 命令

pub mod articles;
pub mod auth;
pub mod comments;
pub mod http;
pub mod searches;
pub mod users;

pub use articles::*;
pub use auth::*;
pub use comments::*;
pub use http::*;
pub use searches::*;
pub use users::*;

fn map_username_write_error(error: sqlx::Error) -> String {
    if error
        .as_database_error()
        .is_some_and(|database_error| database_error.is_unique_violation())
    {
        "用户名已存在".to_string()
    } else {
        format!("Database error: {}", error)
    }
}
