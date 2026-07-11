use sqlx::SqlitePool;
use uuid::Uuid;

use crate::models::user::{AdminUpdateUser, AvatarData, CurrentUser, NewUser, User, UserPublic};

pub async fn insert_user(pool: &SqlitePool, input: &NewUser) -> Result<User, sqlx::Error> {
    let id = Uuid::now_v7().to_string();
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, password, identity)
        VALUES (?, ?, ?, ?)
        RETURNING id, username, password, identity
        "#,
    )
    .bind(id)
    .bind(&input.username)
    .bind(&input.password)
    .bind(&input.identity)
    .fetch_one(pool)
    .await
}

pub async fn find_user_by_username(
    pool: &SqlitePool,
    username: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password, identity FROM users WHERE username = ? LIMIT 1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

pub async fn find_user_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT id, username, password, identity FROM users WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_current_user(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<CurrentUser>, sqlx::Error> {
    sqlx::query_as::<_, CurrentUser>(
        r#"
        SELECT id, username, identity, signature,
               CASE WHEN avatar IS NULL THEN NULL ELSE '/api/users/' || id || '/avatar' END AS avatar_url
        FROM users WHERE id = ? LIMIT 1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn list_users(pool: &SqlitePool, limit: i32) -> Result<Vec<UserPublic>, sqlx::Error> {
    sqlx::query_as::<_, UserPublic>("SELECT id, username, identity FROM users ORDER BY id LIMIT ?")
        .bind(limit.clamp(1, 100))
        .fetch_all(pool)
        .await
}

pub async fn delete_user_by_id(pool: &SqlitePool, id: &str) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected())
}

pub async fn update_user(pool: &SqlitePool, input: AdminUpdateUser) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query(
        r#"
        UPDATE users
        SET username = COALESCE(?, username),
            password = COALESCE(?, password),
            identity = COALESCE(?, identity)
        WHERE id = ?
        "#,
    )
    .bind(input.username)
    .bind(input.password)
    .bind(input.identity)
    .bind(input.user_id)
    .execute(pool)
    .await?
    .rows_affected())
}

pub async fn update_signature(
    pool: &SqlitePool,
    user_id: &str,
    signature: &str,
) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query("UPDATE users SET signature = ? WHERE id = ?")
        .bind(signature)
        .bind(user_id)
        .execute(pool)
        .await?
        .rows_affected())
}

pub async fn save_avatar(
    pool: &SqlitePool,
    user_id: &str,
    avatar: AvatarData,
) -> Result<u64, sqlx::Error> {
    Ok(
        sqlx::query("UPDATE users SET avatar = ?, avatar_content_type = ? WHERE id = ?")
            .bind(avatar.bytes)
            .bind(avatar.content_type)
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

pub async fn remove_avatar(pool: &SqlitePool, user_id: &str) -> Result<u64, sqlx::Error> {
    Ok(
        sqlx::query("UPDATE users SET avatar = NULL, avatar_content_type = NULL WHERE id = ?")
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

pub async fn find_avatar(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<AvatarData>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Vec<u8>, String)>(
        "SELECT avatar, avatar_content_type FROM users WHERE id = ? AND avatar IS NOT NULL",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(bytes, content_type)| AvatarData {
        bytes,
        content_type,
    }))
}

pub async fn get_identity(pool: &SqlitePool, user_id: &str) -> Result<String, sqlx::Error> {
    sqlx::query_scalar("SELECT identity FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await
}
