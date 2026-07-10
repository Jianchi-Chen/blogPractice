//! 搜索相关命令

use crate::repositories::search::*;
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Serialize)]
pub struct SuggestResponse {
    item: Vec<TmpSuggest>,
}

/// 获取搜索建议
#[tauri::command]
pub async fn get_suggestions(
    keyword: String,
    pool: State<'_, SqlitePool>,
) -> Result<SuggestResponse, String> {
    let suggestions = get_suggests_by_keyword(pool.inner(), &keyword)
        .await
        .map_err(|e| format!("Failed to fetch suggestions: {}", e))?;

    Ok(SuggestResponse { item: suggestions })
}
