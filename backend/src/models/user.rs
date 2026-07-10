//! User 模型与持久化操作（Repository）
//! 说明：将数据访问与业务/路由解耦，便于测试与复用。

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::routes::users::AdminEditAccountPayload;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    /// 为简化与 SQLite datetime TEXT 的映射，这里使用 String
    pub identity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserPublic {
    pub id: String,
    pub username: String,
    pub identity: String,
}

impl From<User> for UserPublic {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            identity: u.identity,
        }
    }
}

/// 用户传来的数据
#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub identity: String,
}

/// 新增用户
pub async fn insert_common_user(pool: &SqlitePool, new: &NewUser) -> Result<User, sqlx::Error> {
    let id = Uuid::now_v7().to_string(); // 相比new(), now()可以调用当前时间
    // query_as 是 sqlx 的宏：它在 编译期 检查 SQL 语法，并把结果行直接 按列名映射 到你指定的结构体 User。
    // 第一个类型参数 _ 让编译器推断数据库驱动（这里是 SQLite，只有一种数据库的话可以自己推导）；第二个 User 指定目标结构体。
    sqlx::query_as::<_, User>(
        // r#"..."# Rust原始字符串(raw string)语法，被包裹内容不会被转义
        r#"
        INSERT INTO users (id, username, password, identity)
        VALUES (?, ?, ?, ?)
        RETURNING id, username, password, identity
        "#,
    )
    .bind(&id) // 需要uuid的feature
    .bind(&new.username)
    .bind(&new.password)
    .bind(&new.identity)
    .fetch_one(pool) //执行语句，并 等待一行结果
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
    .fetch_optional(pool) // 执行语句，允许一行或零行
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
    let rows = sqlx::query!(
        r#"SELECT id, username, identity FROM users ORDER BY id LIMIT ?"#,
        limit
    )
    .fetch_all(pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|row| UserPublic {
            id: row.id.unwrap_or_default(),
            username: row.username.unwrap_or_default(),
            identity: row.identity.unwrap_or_default(),
        })
        .collect::<Vec<UserPublic>>();

    Ok(users)
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
    sqlx::query!(
        r#"
            UPDATE users
            SET
            username = COALESCE($1, username),
            password = COALESCE($2, password),
            identity = COALESCE($3, identity)
            WHERE id = $4
    "#,
        new_data.edited_username,
        new_data.edited_password,
        new_data.edited_identity,
        new_data.edited_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
