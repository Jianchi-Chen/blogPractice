//! 统一错误类型：将业务/系统错误映射为 HTTP 响应，便于 handler 中使用 `?`。

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("insufficient privileges")]
    Forbidden,
    #[error(transparent)]
    // #[from] 帮你 自动实现 From<sqlx::Error> for AppError，于是你可以直接 ? 把 sqlx::Error 转成 AppError。
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error("密码哈希错误")]
    PasswordHash(#[from] argon2::password_hash::Error),
    #[error("网络服务错误: {0}")]
    InternalServerError(String),
}

#[derive(Serialize)]
struct ErrorBody {
    code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Sqlx(_) | AppError::Migrate(_) | AppError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".into(),
            ),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Jwt(_) | AppError::PasswordHash(_) => {
                (StatusCode::UNAUTHORIZED, "invalid token".into())
            }
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
        };

        let body = Json(ErrorBody {
            code: status.as_u16(),
            message,
        });

        (status, body).into_response()
    }
}

/// 统一结果类型，简化返回
pub type AppResult<T> = Result<T, AppError>;

pub fn map_username_write_error(error: sqlx::Error) -> AppError {
    if error
        .as_database_error()
        .is_some_and(|database_error| database_error.is_unique_violation())
    {
        AppError::BadRequest("username already registered".into())
    } else {
        AppError::Sqlx(error)
    }
}
