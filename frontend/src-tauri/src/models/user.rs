//! User 模型定义

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub identity: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
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

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub identity: String,
}
