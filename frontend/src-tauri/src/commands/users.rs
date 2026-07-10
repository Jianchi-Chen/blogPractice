//! 用户管理相关命令

use crate::auth::{decode_token, hash_password};
use crate::config::Config;
use crate::models::user::UserPublic;
use crate::models::ResponseMessage;
use crate::repositories::user::{
    delete_user_by_id, edit_user_account, get_ident_by_id, list_users, AdminEditAccountPayload,
};
use crate::tray::update_system_tray_icon;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

use super::map_username_write_error;

#[derive(Serialize)]
pub struct ListUsersResponse {
    users: Vec<UserPublic>,
}

/// 获取用户列表（需要管理员权限）
#[tauri::command]
pub async fn get_users(
    token: String,
    limit: Option<i32>,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<ListUsersResponse, String> {
    // 验证 token
    let claims = decode_token(&config, &token).map_err(|e| format!("Invalid token: {}", e))?;

    // 检查用户身份
    let identity = get_ident_by_id(pool.inner(), &claims.user_id)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if identity != "admin" {
        return Err("Only admin can view users".to_string());
    }

    let default_limit = limit.unwrap_or(10);
    let users = list_users(pool.inner(), default_limit)
        .await
        .map_err(|e| format!("Failed to fetch users: {}", e))?;

    Ok(ListUsersResponse { users })
}

/// 删除用户（需要管理员权限）
#[tauri::command]
pub async fn delete_user(
    token: String,
    user_id: String,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<ResponseMessage, String> {
    // 验证 token
    let claims = decode_token(&config, &token).map_err(|e| format!("Invalid token: {}", e))?;

    // 检查用户身份
    let identity = get_ident_by_id(pool.inner(), &claims.user_id)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if identity != "admin" {
        return Err("Only admin can delete users".to_string());
    }

    if user_id == "1" {
        return Err("Cannot delete the superadmin account".to_string());
    }

    let rows_affected = delete_user_by_id(pool.inner(), &user_id)
        .await
        .map_err(|e| format!("Failed to delete user: {}", e))?;
    if rows_affected == 0 {
        return Err("User not found".to_string());
    }

    Ok(ResponseMessage {
        message: "done".to_string(),
    })
}

/// 编辑用户账号（仅管理员可用）
#[tauri::command]
pub async fn edit_account(
    token: String,
    payload: AdminEditAccountPayload,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<ResponseMessage, String> {
    // 验证 token
    let claims = decode_token(&config, &token).map_err(|e| format!("Invalid token: {}", e))?;

    // 检查用户身份
    let identity = get_ident_by_id(pool.inner(), &claims.user_id)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    if identity != "admin" {
        return Err("Only admin can edit user accounts".to_string());
    }

    if let Some(identity) = payload.edited_identity.as_deref() {
        if !matches!(identity, "admin" | "user" | "visitor") {
            return Err("Invalid user identity".to_string());
        }
    }

    // 超级管理员账号必须始终保留管理员身份。
    if payload.edited_id == "1"
        && payload
            .edited_identity
            .as_deref()
            .is_some_and(|identity| identity != "admin")
    {
        return Err("Cannot demote the superadmin account".to_string());
    }

    // 将传递的密码转为 hash
    let new_password = if let Some(ref password) = payload.edited_password {
        if !password.is_empty() {
            Some(hash_password(password).map_err(|e| format!("Failed to hash password: {}", e))?)
        } else {
            None
        }
    } else {
        None
    };

    let updated_payload = AdminEditAccountPayload {
        edited_id: payload.edited_id,
        edited_username: payload.edited_username,
        edited_password: new_password,
        edited_identity: payload.edited_identity,
    };

    edit_user_account(pool.inner(), updated_payload)
        .await
        .map_err(map_username_write_error)?;

    Ok(ResponseMessage {
        message: "done".to_string(),
    })
}

#[derive(Deserialize)]
pub struct SaveAvatarPayload {
    token: String,
    /// 源文件的绝对路径
    source_path: String,
}

#[derive(Serialize)]
pub struct SaveAvatarResponse {
    /// 返回存储的文件路径（相对于 app_data_dir）
    path: String,
}

/// 保存用户头像到本地
#[tauri::command]
pub async fn save_avatar(
    app: AppHandle,
    payload: SaveAvatarPayload,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<SaveAvatarResponse, String> {
    // 验证 token
    let claims =
        decode_token(&config, &payload.token).map_err(|e| format!("Invalid token: {}", e))?;

    // 验证用户是否存在
    get_ident_by_id(pool.inner(), &claims.user_id)
        .await
        .map_err(|e| format!("User not found: {}", e))?;

    // 验证源文件是否存在
    let source_path = PathBuf::from(&payload.source_path);
    if !source_path.exists() {
        return Err("Source file does not exist".to_string());
    }

    // 验证文件类型（通过扩展名）
    let extension = source_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or("File has no extension")?;

    if extension != "png" && extension != "jpg" && extension != "jpeg" {
        return Err("Only PNG and JPG files are allowed".to_string());
    }

    // 统一扩展名
    let normalized_ext = if extension == "jpeg" {
        "jpg"
    } else {
        &extension
    };

    // 获取 app_data_dir 并创建 icons 目录
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let icons_dir = app_data.join("icons");
    fs::create_dir_all(&icons_dir)
        .map_err(|e| format!("Failed to create icons directory: {}", e))?;

    // 生成文件名：user_id + 时间戳
    let timestamp = chrono::Utc::now().timestamp();
    let filename = format!("avatar_{}_{}.{}", claims.user_id, timestamp, normalized_ext);
    let dest_path = icons_dir.join(&filename);

    // 复制文件（而不是移动，保留用户的原始文件）
    fs::copy(&source_path, &dest_path).map_err(|e| format!("Failed to copy avatar: {}", e))?;

    if extension == "png" {
        log::info!("将png文件更新为系统托盘: {:?}", dest_path);
        // 将头像应用至系统托盘(仅限.png)
        update_system_tray_icon(&app, dest_path.to_str().unwrap_or(""))?;
    }

    log::info!("Avatar saved: {:?}", dest_path);

    // 清理旧头像（保留最新 10 个）
    cleanup_old_avatars(&icons_dir, &claims.user_id, 10)
        .map_err(|e| format!("Failed to cleanup old avatars: {}", e))?;

    // 返回相对路径
    let relative_path = format!("icons/{}", filename);

    Ok(SaveAvatarResponse {
        path: relative_path,
    })
}

/// 清理用户的旧头像，只保留最新的 max_keep 个
fn cleanup_old_avatars(icons_dir: &PathBuf, user_id: &str, max_keep: usize) -> Result<(), String> {
    // 读取目录
    let entries =
        fs::read_dir(icons_dir).map_err(|e| format!("Failed to read icons directory: {}", e))?;

    // 收集该用户的所有头像文件
    let mut user_avatars: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
    let prefix = format!("avatar_{}_", user_id);

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.starts_with(&prefix) {
                // 获取文件修改时间
                let metadata =
                    fs::metadata(&path).map_err(|e| format!("Failed to read metadata: {}", e))?;
                let modified = metadata
                    .modified()
                    .map_err(|e| format!("Failed to get modified time: {}", e))?;

                user_avatars.push((path, modified));
            }
        }
    }

    // 如果文件数量超过限制，删除旧的
    if user_avatars.len() > max_keep {
        // 按修改时间排序（旧的在前）
        user_avatars.sort_by_key(|(_, time)| *time);

        // 删除最旧的文件
        let to_delete = user_avatars.len() - max_keep;
        for (path, _) in user_avatars.iter().take(to_delete) {
            fs::remove_file(path).map_err(|e| format!("Failed to delete old avatar: {}", e))?;
            log::info!("Deleted old avatar: {:?}", path);
        }
    }

    Ok(())
}
