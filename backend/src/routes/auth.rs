//! /auth 相关路由：注册、登录、获取当前用户信息、验证token
//! 说明：演示如何组合 models + auth + error + state

use crate::auth::{
    MaybeJwtAuth, decode_token, generate_token, hash_password, require_admin, verify_password,
};
use crate::db::AppState;
use crate::error::{AppError, AppResult};
use crate::models::user::{NewUser, find_user_by_id, find_user_by_username, insert_common_user};
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Deserialize, Clone)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub identity: Option<String>, // 可选字段，仅 admin 创建用户时使用
}

#[derive(Deserialize, Clone, Debug)]
pub struct LoginPayload {
    pub password: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub identity: String,
}

/// POST /register
/// 接受前端注册以及admin页面的新建用户请求
/// 前端注册时不带身份字段，默认注册为普通用户
/// admin创建用户时可带身份字段
pub async fn register(
    State(state): State<Arc<AppState>>,
    MaybeJwtAuth(requester): MaybeJwtAuth,
    Json(payload): Json<RegisterPayload>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    // 简单校验
    if payload.username.trim().is_empty() || payload.password.len() < 3 {
        return Err(AppError::BadRequest(
            "invalid username or password too short".into(),
        ));
    }

    // 检查是否已经存在
    if find_user_by_username(&state.pool, &payload.username)
        .await?
        .is_some()
    {
        tracing::info!("用户名已被注册: {}", payload.username);
        return Err(AppError::BadRequest("username already registered".into()));
    }

    // 公开注册只能创建普通用户，其他身份必须由管理员创建。
    let identity = match payload.identity.as_deref() {
        None | Some("user") => "user".to_string(),
        Some(identity @ ("admin" | "visitor")) => {
            let claims = requester.as_ref().ok_or_else(|| AppError::Forbidden)?;
            require_admin(&state, claims).await?;
            identity.to_string()
        }
        Some(_) => {
            return Err(AppError::BadRequest(
                "invalid identity, must be 'admin', 'user' or 'visitor'".into(),
            ));
        }
    };

    let password_hash = hash_password(&payload.password)?;
    let new = NewUser {
        username: payload.username.clone(),
        password: password_hash,
        identity: identity.clone(),
    };
    let user = insert_common_user(&state.pool, &new).await?;
    let token = generate_token(&state, user.id.clone(), &user.username)?;

    tracing::info!(
        "用户注册成功: {}, 用户身份为: {}",
        payload.username,
        user.identity
    );
    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user_id: user.id,
            username: user.username,
            identity: user.identity,
        }),
    ))
}

/// POST /login
// 签名解析：
// State(state) = “把全局 AppState 拿出来，变量叫 state”；
// Json(payload) = “把请求体 JSON 解析成 LoginPayload，变量叫 payload”。
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginPayload>,
) -> AppResult<Json<AuthResponse>> {
    let Some(user) = find_user_by_username(&state.pool, &payload.username).await? else {
        tracing::warn!("/login: 用户不存在: {}", payload.username);
        return Err(AppError::Unauthorized("invalid credentials".into()));
    };

    if !verify_password(&payload.password, &user.password) {
        tracing::warn!("/login: 密码错误 for user: {}", payload.username);
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    tracing::info!("/login: user authenticated: {}", payload.username);

    let token = generate_token(&state, user.id.clone(), &user.username)?;
    Ok(Json(AuthResponse {
        token,
        user_id: user.id,
        username: user.username,
        identity: user.identity,
    }))
}

// 身份验证
pub async fn get_ident_by_id(pool: &SqlitePool, id: &str) -> Result<String, sqlx::Error> {
    let ident = match find_user_by_id(pool, id.to_string()).await? {
        Some(v) => {
            tracing::warn!("found user by id: {}, identity: {}", id, v.identity);
            v.identity
        }
        None => {
            tracing::warn!("can't find user by id: {}", id);
            "visitor".to_string()
        }
    };
    Ok(ident)
}

/// 验证 token
#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct VerifyTokenResponse {
    pub user_id: String,
    pub message: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn verify_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<VerifyTokenRequest>,
) -> AppResult<Json<VerifyTokenResponse>> {
    let claims = decode_token(&state, &payload.token)
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    Ok(Json(VerifyTokenResponse {
        user_id: claims.user_id,
        message: claims.message,
        exp: claims.exp,
        iat: claims.iat,
    }))
}

/// 获取当前用户信息
#[derive(Deserialize)]
pub struct GetCurrentUserRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct GetCurrentUserResponse {
    pub id: String,
    pub username: String,
    pub identity: String,
}

pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GetCurrentUserRequest>,
) -> AppResult<Json<GetCurrentUserResponse>> {
    // 验证 token
    let claims = decode_token(&state, &payload.token)
        .map_err(|_| AppError::Unauthorized("Invalid token".into()))?;

    // 查询用户信息
    let user = find_user_by_id(&state.pool, claims.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound)?;

    Ok(Json(GetCurrentUserResponse {
        id: user.id,
        username: user.username,
        identity: user.identity,
    }))
}
