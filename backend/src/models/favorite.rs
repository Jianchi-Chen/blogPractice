use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FavoriteState {
    pub article_id: String,
    pub favorited: bool,
}
