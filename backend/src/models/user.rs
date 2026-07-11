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

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CurrentUser {
    pub id: String,
    pub username: String,
    pub identity: String,
    pub signature: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub identity: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdminUpdateUser {
    #[serde(alias = "edited_id")]
    pub user_id: String,
    #[serde(alias = "edited_username")]
    pub username: Option<String>,
    #[serde(alias = "edited_password")]
    pub password: Option<String>,
    #[serde(alias = "edited_identity")]
    pub identity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileInput {
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct AvatarData {
    pub bytes: Vec<u8>,
    pub content_type: String,
}
