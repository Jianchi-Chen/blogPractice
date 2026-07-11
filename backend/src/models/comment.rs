use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Serialize, FromRow, Deserialize, Debug)]
pub struct Comment {
    pub comment_id: String,
    pub article_id: Option<String>,
    pub user: Option<String>,
    pub content: Option<String>,
    pub created_at: Option<String>,
    pub parent_id: Option<String>,
    pub like_count: Option<i64>,
}

#[derive(Clone, Serialize, FromRow, Deserialize, Debug)]
pub struct CommentWithLike {
    pub comment_id: String,
    pub article_id: Option<String>,
    pub user: Option<String>,
    pub content: Option<String>,
    pub created_at: Option<String>,
    pub parent_id: Option<String>,
    pub like_count: Option<i64>,
    pub liked_by_me: Option<i64>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct CreateCommentInput {
    pub content: String,
    pub parent_id: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct SetCommentLikeInput {
    pub liked: bool,
}

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
pub struct CommentLikeState {
    pub comment_id: String,
    pub liked_by_me: bool,
    pub like_count: i64,
}
