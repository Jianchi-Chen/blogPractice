//! User Repository - 用户数据访问层

use serde::Deserialize;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::user::{NewUser, User, UserPublic};

#[derive(Deserialize, Debug)]
pub struct AdminEditAccountPayload {
    pub edited_id: String,
    pub edited_username: Option<String>,
    pub edited_password: Option<String>,
    pub edited_identity: Option<String>,
}

/// 新增用户
pub async fn insert_common_user(pool: &SqlitePool, new: &NewUser) -> Result<User, sqlx::Error> {
    let id = Uuid::now_v7().to_string();

    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, password, identity)
        VALUES (?, ?, ?, ?)
        RETURNING id, username, password, identity
        "#,
    )
    .bind(&id)
    .bind(&new.username)
    .bind(&new.password)
    .bind(&new.identity)
    .fetch_one(pool)
    .await
}

/// 通过用户名查找用户
pub async fn find_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"SELECT id, username, password, identity FROM users WHERE username = ? LIMIT 1"#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

/// 通过id查找用户
pub async fn find_user_by_id(pool: &SqlitePool, id: String) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"SELECT id, username, password, identity FROM users WHERE id = ? LIMIT 1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// 用户列表
pub async fn list_users(pool: &SqlitePool, limit: i32) -> Result<Vec<UserPublic>, sqlx::Error> {
    sqlx::query_as::<_, UserPublic>(
        r#"SELECT id, username, identity FROM users ORDER BY id LIMIT ?"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// 通过id删除用户
pub async fn delete_user_by_id(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(r#"DELETE FROM users WHERE id = ?"#)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// 编辑用户账号
pub async fn edit_user_account(
    pool: &SqlitePool,
    new_data: AdminEditAccountPayload,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE users
        SET
            username = COALESCE($1, username),
            password = COALESCE($2, password),
            identity = COALESCE($3, identity)
        WHERE id = $4
        "#,
    )
    .bind(new_data.edited_username)
    .bind(new_data.edited_password)
    .bind(new_data.edited_identity)
    .bind(new_data.edited_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// 通过 ID 获取用户身份
pub async fn get_ident_by_id(pool: &SqlitePool, user_id: &str) -> Result<String, sqlx::Error> {
    sqlx::query_scalar(r#"SELECT identity FROM users WHERE id = ?"#)
        .bind(user_id)
        .fetch_one(pool)
        .await
}
