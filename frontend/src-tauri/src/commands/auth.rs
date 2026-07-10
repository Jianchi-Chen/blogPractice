//! 认证相关命令

use crate::auth::{
    decode_token, generate_token, hash_password, require_admin, verify_password, Claims,
};
use crate::config::Config;
use crate::models::user::NewUser;
use crate::repositories::user::{find_user_by_username, insert_common_user};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::State;

use super::map_username_write_error;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub identity: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub identity: Option<String>,
}

/// 登录命令
#[tauri::command]
pub async fn login(
    credentials: LoginRequest,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<LoginResponse, String> {
    log::info!("login attempt");

    // 查询用户
    let user = match find_user_by_username(pool.inner(), &credentials.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            log::error!("login failed - user not found");
            return Err("未注册用户".to_string());
        }
        Err(e) => {
            log::error!("login failed due to database error: {}", e);
            return Err(format!("Database error: {}", e));
        }
    };

    // 验证密码
    if !verify_password(&credentials.password, &user.password) {
        log::error!("login failed due to incorrect password");
        return Err("用户名或密码错误".to_string());
    }

    // 生成 token
    let token = generate_token(&config, user.id.clone(), &user.username).map_err(|e| {
        log::error!("login failed due to token generation error: {}", e);
        format!("Failed to generate token: {}", e)
    })?;

    log::info!("login successful");

    Ok(LoginResponse {
        token,
        user_id: user.id,
        username: user.username,
        identity: user.identity,
    })
}

/// 注册命令
#[tauri::command]
pub async fn register(
    user_info: RegisterRequest,
    token: Option<String>,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<LoginResponse, String> {
    if user_info.username.trim().is_empty() || user_info.password.len() < 3 {
        return Err("用户名不能为空且密码长度不能少于 3 位".to_string());
    }

    // 检查用户名是否已存在
    let existing = find_user_by_username(pool.inner(), &user_info.username)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if existing.is_some() {
        log::warn!("register attempt with existing username");
        return Err("用户名已存在".to_string());
    }

    let identity = match user_info.identity.as_deref() {
        None | Some("user") => "user".to_string(),
        Some(identity @ ("admin" | "visitor")) => {
            let token = token
                .as_deref()
                .ok_or_else(|| "Admin privileges required".to_string())?;
            require_admin(&config, pool.inner(), token).await?;
            identity.to_string()
        }
        Some(_) => return Err("无效的用户身份".to_string()),
    };

    // 哈希密码
    let password_hash = hash_password(&user_info.password)
        .map_err(|e| format!("Failed to hash password: {}", e))?;

    // 创建新用户
    let new_user = NewUser {
        username: user_info.username.clone(),
        password: password_hash,
        identity,
    };

    let user = insert_common_user(pool.inner(), &new_user)
        .await
        .map_err(map_username_write_error)?;

    // 生成 token
    let token = generate_token(&config, user.id.clone(), &user.username)
        .map_err(|e| format!("Failed to generate token: {}", e))?;

    log::info!("User registered successfully");

    Ok(LoginResponse {
        token,
        user_id: user.id,
        username: user.username,
        identity: user.identity,
    })
}

/// 验证 token 命令
#[tauri::command]
pub async fn verify_token(token: String, config: State<'_, Config>) -> Result<Claims, String> {
    decode_token(&config, &token).map_err(|e| format!("Invalid token: {}", e))
}

/// 获取当前用户信息
#[tauri::command]
pub async fn get_current_user(
    token: String,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<serde_json::Value, String> {
    // 验证 token
    let claims = decode_token(&config, &token).map_err(|e| format!("Invalid token: {}", e))?;

    // 查询用户信息
    let user = sqlx::query!(
        r#"
        SELECT id, username, identity
        FROM users
        WHERE id = ?
        "#,
        claims.user_id
    )
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or("User not found")?;

    Ok(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "identity": user.identity,
    }))
}
