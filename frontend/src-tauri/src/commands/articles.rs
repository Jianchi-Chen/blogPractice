//! 文章相关命令

use crate::auth::{optional_token_is_admin, require_admin};
use crate::config::Config;
use crate::models::article::{ArticleModel, PubArticles};
use crate::models::ResponseMessage;
use crate::repositories::article;
use serde::Serialize;
use sqlx::SqlitePool;
use tauri::State;

#[derive(Serialize, Debug)]
pub struct ArticleResponse {
    articles: Vec<PubArticles>,
}

/// 获取文章列表
#[tauri::command]
pub async fn get_articles(
    condition: Option<String>,
    token: Option<String>,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<ArticleResponse, String> {
    log::info!("attempt to get_articles");
    let params = article::GetArticlesParams { condition };
    let include_unpublished =
        optional_token_is_admin(&config, pool.inner(), token.as_deref()).await?;

    let articles = article::get_articles(pool.inner(), params, include_unpublished)
        .await
        .map_err(|e| format!("Failed to fetch articles: {}", e))?;

    log::info!("success get_articles");
    Ok(ArticleResponse { articles })
}

/// 获取单篇文章
#[tauri::command]
pub async fn get_article_by_id(
    id: String,
    token: Option<String>,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<ArticleModel, String> {
    log::info!("attempt to get_article_by_id");
    let result = article::find_article_by_id(pool.inner(), &id)
        .await
        .map_err(|e| format!("Failed to fetch article: {}", e))?
        .ok_or("Article not found")?;

    let can_view_unpublished =
        optional_token_is_admin(&config, pool.inner(), token.as_deref()).await?;
    if result.status.as_deref() != Some("published") && !can_view_unpublished {
        return Err("Article not found".to_string());
    }

    log::info!("success get_article_by_id");
    Ok(result)
}

/// 新建文章
#[tauri::command]
pub async fn create_article(
    token: String,
    article_data: article::NewArticle,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<ArticleModel, String> {
    log::info!("attempt to create_article");
    require_admin(&config, pool.inner(), &token).await?;

    let result = article::post_article(pool.inner(), &article_data)
        .await
        .map_err(|e| format!("Failed to create article: {}", e))?;

    log::info!("success create_article");
    Ok(ArticleModel {
        message: "done".to_string(),
        ..result
    })
}

/// 更新文章
#[tauri::command]
pub async fn update_article(
    token: String,
    id: String,
    article_data: article::NewArticle,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<ArticleModel, String> {
    log::info!("attempt to update_article");
    require_admin(&config, pool.inner(), &token).await?;

    let result = article::put_article_by_id(pool.inner(), &id, article_data)
        .await
        .map_err(|e| format!("Failed to update article: {}", e))?;

    log::info!("success update_article");
    Ok(ArticleModel {
        message: "done".to_string(),
        ..result
    })
}

/// 删除文章
#[tauri::command]
pub async fn delete_article(
    token: String,
    id: String,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<ResponseMessage, String> {
    log::info!("attempt to delete_article");
    require_admin(&config, pool.inner(), &token).await?;

    let rows_affected = article::delete_article_by_id(pool.inner(), &id)
        .await
        .map_err(|e| format!("Failed to delete article: {}", e))?;
    if rows_affected == 0 {
        return Err("Article not found".to_string());
    }

    log::info!("success delete_article");
    Ok(ResponseMessage {
        message: "done".to_string(),
    })
}

/// 更改文章状态
#[tauri::command]
pub async fn toggle_article_status(
    token: String,
    id: String,
    status: article::NewStatus,
    pool: State<'_, SqlitePool>,
    config: State<'_, Config>,
) -> Result<ArticleModel, String> {
    log::info!("attemp to toggle_article_status");
    require_admin(&config, pool.inner(), &token).await?;

    if !matches!(status.toggle.as_str(), "draft" | "published" | "archived") {
        return Err("Invalid article status".to_string());
    }

    let result = article::patch_article_by_id(pool.inner(), &id, status)
        .await
        .map_err(|e| {
            log::error!("Failed to toggle article status: {}", e);
            format!("Failed to toggle article status: {}", e)
        })?;

    log::info!("success toggle_article_status");
    Ok(result)
}
