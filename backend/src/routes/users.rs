use axum::{
    Json,
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    auth::{Claims, JwtAuth, hash_password, require_admin},
    db::AppState,
    error::{AppError, AppResult, map_username_write_error},
    models::user::{AdminUpdateUser, AvatarData, CurrentUser, UpdateProfileInput, UserPublic},
    repositories::user::{
        delete_user_by_id, find_avatar, find_current_user, list_users, remove_avatar, save_avatar,
        update_signature, update_user,
    },
};

const MAX_AVATAR_BYTES: usize = 2 * 1024 * 1024;

#[derive(Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserPublic>,
}

#[derive(Deserialize, Debug)]
pub struct UserListQuery {
    pub limit: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateUserInput {
    pub username: Option<String>,
    pub password: Option<String>,
    pub identity: Option<String>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Query(query): Query<UserListQuery>,
) -> AppResult<Json<UserListResponse>> {
    require_admin(&state, &claims).await?;
    let users = list_users(&state.pool, query.limit.unwrap_or(10)).await?;
    Ok(Json(UserListResponse { users }))
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    JwtAuth(claims): JwtAuth,
) -> AppResult<StatusCode> {
    require_admin(&state, &claims).await?;
    if user_id == "1" {
        return Err(AppError::BadRequest(
            "cannot delete the superadmin account".into(),
        ));
    }
    if delete_user_by_id(&state.pool, &user_id).await? == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
    JwtAuth(claims): JwtAuth,
    Json(input): Json<UpdateUserInput>,
) -> AppResult<StatusCode> {
    update_account(&state, &claims, user_id, input).await
}

pub(crate) async fn update_account(
    state: &AppState,
    claims: &Claims,
    user_id: String,
    input: UpdateUserInput,
) -> AppResult<StatusCode> {
    require_admin(state, claims).await?;
    let update = prepare_update(user_id, input)?;
    if update_user(&state.pool, update)
        .await
        .map_err(map_username_write_error)?
        == 0
    {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    Json(input): Json<UpdateProfileInput>,
) -> AppResult<Json<CurrentUser>> {
    let signature = input.signature.trim();
    if signature.chars().count() > 120 {
        return Err(AppError::BadRequest(
            "signature must not exceed 120 characters".into(),
        ));
    }
    if update_signature(&state.pool, &claims.user_id, signature).await? == 0 {
        return Err(AppError::NotFound);
    }
    current_user_response(&state, &claims.user_id).await
}

pub async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
    mut multipart: Multipart,
) -> AppResult<Json<CurrentUser>> {
    let mut avatar = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| AppError::BadRequest(error.to_string()))?
    {
        if field.name() != Some("avatar") {
            continue;
        }
        let content_type = field
            .content_type()
            .map(str::to_string)
            .ok_or_else(|| AppError::BadRequest("avatar content type is required".into()))?;
        let bytes = field
            .bytes()
            .await
            .map_err(|error| AppError::BadRequest(error.to_string()))?;
        validate_avatar(&content_type, &bytes)?;
        avatar = Some(AvatarData {
            bytes: bytes.to_vec(),
            content_type,
        });
        break;
    }

    let avatar = avatar.ok_or_else(|| AppError::BadRequest("avatar file is required".into()))?;
    if save_avatar(&state.pool, &claims.user_id, avatar).await? == 0 {
        return Err(AppError::NotFound);
    }
    current_user_response(&state, &claims.user_id).await
}

pub async fn delete_avatar(
    State(state): State<Arc<AppState>>,
    JwtAuth(claims): JwtAuth,
) -> AppResult<Json<CurrentUser>> {
    if remove_avatar(&state.pool, &claims.user_id).await? == 0 {
        return Err(AppError::NotFound);
    }
    current_user_response(&state, &claims.user_id).await
}

pub async fn avatar(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> AppResult<Response> {
    let avatar = find_avatar(&state.pool, &user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let content_type = HeaderValue::from_str(&avatar.content_type)
        .map_err(|_| AppError::InternalServerError("invalid avatar content type".into()))?;
    Ok((
        [(header::CONTENT_TYPE, content_type)],
        Body::from(avatar.bytes),
    )
        .into_response())
}

fn prepare_update(user_id: String, input: UpdateUserInput) -> AppResult<AdminUpdateUser> {
    if let Some(identity) = input.identity.as_deref()
        && !matches!(identity, "admin" | "user" | "visitor")
    {
        return Err(AppError::BadRequest("invalid identity".into()));
    }
    ensure_superadmin_identity(&user_id, input.identity.as_deref())?;
    let password = input
        .password
        .filter(|password| !password.is_empty())
        .map(|password| hash_password(&password))
        .transpose()?;
    Ok(AdminUpdateUser {
        user_id,
        username: input
            .username
            .filter(|username| !username.trim().is_empty()),
        password,
        identity: input.identity,
    })
}

fn ensure_superadmin_identity(user_id: &str, identity: Option<&str>) -> AppResult<()> {
    if user_id == "1" && identity.is_some_and(|identity| identity != "admin") {
        return Err(AppError::BadRequest(
            "cannot demote the superadmin account".into(),
        ));
    }
    Ok(())
}

fn validate_avatar(content_type: &str, bytes: &[u8]) -> AppResult<()> {
    if bytes.is_empty() || bytes.len() > MAX_AVATAR_BYTES {
        return Err(AppError::BadRequest(
            "avatar must be between 1 byte and 2 MiB".into(),
        ));
    }
    let valid = match content_type {
        "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        _ => false,
    };
    if !valid {
        return Err(AppError::BadRequest(
            "only valid PNG and JPEG avatars are supported".into(),
        ));
    }
    Ok(())
}

async fn current_user_response(state: &AppState, user_id: &str) -> AppResult<Json<CurrentUser>> {
    find_current_user(&state.pool, user_id)
        .await?
        .map(Json)
        .ok_or(AppError::NotFound)
}
