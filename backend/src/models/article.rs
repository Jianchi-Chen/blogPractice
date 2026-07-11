use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Article {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub created_at: Option<String>,
    pub update_at: Option<String>,
    pub status: Option<String>,
    pub views: Option<i32>,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ArticleSummary {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub created_at: String,
    pub status: String,
    pub views: i32,
    pub tags: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticleInput {
    pub id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArticleStatusInput {
    #[serde(alias = "toggle")]
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ArticleListQuery {
    #[serde(alias = "condition")]
    pub query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ArticleSuggestion {
    pub id: String,
    pub title: String,
}
