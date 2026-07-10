//! /users 相关路由：列表（受保护）

use crate::auth::{JwtAuth, hash_password};
use crate::db::AppState;
use crate::error::{AppError, AppResult};
use crate::models::user::{UserPublic, delete_user_by_id, edit_user_account, list_users};
use crate::routes::auth::get_ident_by_id;
use axum::extract::{Path, Query};
use axum::{Json, extract::State};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct ListUsersResponse {
    users: Vec<UserPublic>,
}

// HTTP层函数，对外接口
pub async fn get_users(
    State(state): State<Arc<AppState>>,
    JwtAuth(auth): JwtAuth,
    Query(query): Query<UsersQuery>,
) -> AppResult<Json<ListUsersResponse>> {
    tracing::info!("Received query: {:?}", query);

    // 验证用户身份
    if get_ident_by_id(&state.pool, &auth.user_id).await? != "admin" {
        tracing::warn!("Unauthorized get_users attempt by user: {:?}", auth.user_id);
        return Err(AppError::Unauthorized("only admin can view users".into()));
    }

    let default_limit = query.limit.unwrap_or(10);

    let users = list_users(&state.pool, default_limit).await?;
    tracing::info!("Fetched users: {:?}", users);
    Ok(Json(ListUsersResponse { users }))
}

pub async fn delete_users(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    JwtAuth(auth): JwtAuth,
) -> AppResult<StatusCode> {
    tracing::info!("Received request to delete user: {:?}", user_id);

    // 验证前端传来的token
    if get_ident_by_id(&state.pool, &auth.user_id).await? != "admin" {
        tracing::warn!("Unauthorized delete attempt by user: {:?}", auth.user_id);
        return Err(crate::error::AppError::Unauthorized(
            "only admin can delete users".into(),
        ));
    }

    if user_id == "1" {
        return Err(AppError::BadRequest(
            "cannot delete the superadmin account".into(),
        ));
    }

    delete_user_by_id(&state.pool, &user_id).await?;
    tracing::info!("Deleted user: {:?}", user_id);

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, Debug)]
pub struct UsersQuery {
    pub limit: Option<i32>,
}

// 编辑用户账号（仅管理员可用）
pub async fn edit_account(
    State(state): State<Arc<AppState>>,
    JwtAuth(auth): JwtAuth,
    Json(payload): Json<AdminEditAccountPayload>,
) -> AppResult<StatusCode> {
    tracing::info!(
        "AdminEditAccountPayload() Received request to edit account: {:?}",
        auth.user_id
    );

    // 仅管理员编辑
    if get_ident_by_id(&state.pool, &auth.user_id).await? != "admin" {
        tracing::warn!(
            "Unauthorized edit attempt by user: {:?}, current identity: {:?}",
            auth.user_id,
            payload.edited_identity
        );
        return Err(AppError::Unauthorized(
            "cannot edit other user's account".into(),
        ));
    }

    if let Some(identity) = payload.edited_identity.as_deref()
        && !matches!(identity, "admin" | "user" | "visitor")
    {
        return Err(AppError::BadRequest("invalid identity".into()));
    }

    // 超级管理员账号必须始终保留管理员身份。
    if payload.edited_id == "1"
        && payload
            .edited_identity
            .as_deref()
            .is_some_and(|identity| identity != "admin")
    {
        tracing::warn!("Attempt to demote superadmin by user: {:?}", auth.user_id);
        return Err(AppError::BadRequest(
            "cannot change superadmin identity".into(),
        ));
    }

    // 将传递的密码转为hash
    let new_password = if let Some(ref password) = payload.edited_password
        && !password.is_empty()
    {
        match hash_password(password) {
            Ok(hashed) => Some(hashed),
            Err(e) => {
                tracing::warn!(
                    "failed to hash password in edit account request by user: {:?}, error: {:?}",
                    auth.user_id,
                    e
                );
                return Err(AppError::InternalServerError(
                    "failed to hash password".into(),
                ));
            }
        }
    } else {
        None
    };
    let payload = AdminEditAccountPayload {
        edited_id: payload.edited_id,
        edited_username: payload.edited_username,
        edited_password: new_password,
        edited_identity: payload.edited_identity,
    };

    edit_user_account(&state.pool, payload).await?;

    tracing::info!("Edited account for user: {:?}", auth.user_id);

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Debug)]
pub struct AdminEditAccountPayload {
    pub edited_id: String,
    pub edited_username: Option<String>,
    pub edited_password: Option<String>,
    pub edited_identity: Option<String>,
}
