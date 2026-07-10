//! 鉴权模块：
//! - JWT 生成/校验（基于 jsonwebtoken）
//! - 密码哈希/校验（基于 argon2）
//! - `JwtAuth` 提取器：从请求头解析 Bearer Token 并验证，向 handler 提供 Claims

use crate::db::AppState;
use crate::error::{AppError, AppResult};
use crate::models::user::find_user_by_id;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand::rngs::OsRng;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: String, // user id
    pub message: String, // 冗余字段，便于调试展示
    pub exp: usize,      // 过期时间（秒）
    pub iat: usize,      // 签发时间（秒）
}

fn now_ts() -> usize {
    Utc::now().timestamp().max(0) as usize
}

pub fn generate_token(state: &AppState, user_id: String, username: &str) -> AppResult<String> {
    let iat = now_ts();
    let exp = (Utc::now() + Duration::seconds(state.cfg.jwt_ttl))
        .timestamp()
        .max(0) as usize;

    let claims = Claims {
        user_id,
        message: username.to_string(),
        exp,
        iat,
    };
    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.cfg.jwt_secret.as_bytes()),
    )?;
    Ok(token)
}

/// 统一封装密码哈希
pub fn hash_password(plain: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(plain.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// 校验密码
pub fn verify_password(plain: &str, hashed: &str) -> bool {
    if let Ok(parsed) = PasswordHash::new(hashed) {
        Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok()
    } else {
        false
    }
}

/// 基于 Bearer Token 的鉴权提取器（强制登录器）
/// 用法：在 handler 参数中写 `JwtAuth(claims): JwtAuth`
pub struct JwtAuth(pub Claims);
// #[async_trait] // 同步trait
impl<S> axum::extract::FromRequestParts<S> for JwtAuth
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;
    // 强制登录，否则抛错
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        /* 获取全局状态（Arc<AppState>）——由 Router::with_state 注入
        let state = parts
            .extensions
            .get::<Arc<AppState>>()
            .cloned()
            .ok_or_else(|| AppError::BadRequest("JwtAuth missing app state".into()))?;
        */

        // 泛型提取方式
        let state = Arc::<AppState>::from_ref(state); // ✅ 正确提取方式

        // 解析 Authorization: Bearer <token>
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|hv| hv.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

        /* debug版；解析Authorization
                for (key, value) in parts.headers.iter() {
            println!("Header: {} = {:?}", key, value);
        }
        let header_value = parts.headers.get(AUTHORIZATION);
        println!("header_value: {:?}", header_value);
        let header_str = header_value.and_then(|hv| hv.to_str().ok());
        println!("header_str: {:?}", header_str);
        let token_opt = header_str.and_then(|v| v.strip_prefix("Bearer "));
        println!("token_opt: {:?}", token_opt);
        let token =
            token_opt.ok_or_else(|| AppError::Unauthorized("missing bearer token".into()))?;

        */

        let claims = decode_token(&state, token)?;
        // println!("debug JwtAuth claims.user_id: {}", &claims.user_id);

        Ok(JwtAuth(claims))
    }
}

/// 用 state 里的密钥验证签名。
pub fn decode_token(state: &AppState, token: &str) -> AppResult<Claims> {
    // println!("debug decode_token:{}", token);
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.cfg.jwt_secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(data.claims)
}

pub async fn is_admin(state: &AppState, user_id: &str) -> AppResult<bool> {
    let user = find_user_by_id(&state.pool, user_id.to_string()).await?;
    Ok(user.is_some_and(|user| user.identity == "admin"))
}

pub async fn require_admin(state: &AppState, claims: &Claims) -> AppResult<()> {
    if is_admin(state, &claims.user_id).await? {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

// 可选登录器
pub struct MaybeJwtAuth(pub Option<Claims>);
impl<S> FromRequestParts<S> for MaybeJwtAuth
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    // 永不失败
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = Arc::<AppState>::from_ref(state);

        let maybe_claims = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|hv| hv.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .and_then(|tok| decode_token(&state, tok).ok());

        Ok(MaybeJwtAuth(maybe_claims))
    }
}
