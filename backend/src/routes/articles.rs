use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
// use sqlx::types::Json;

use crate::{
    auth::{JwtAuth, MaybeJwtAuth, is_admin, require_admin},
    db::AppState,
    error::{AppError, AppResult},
    models::{
        article::{
            ArticleModel, PubArticles, delete_article_by_id, find_article_by_id, get_articles,
            patch_article_by_id, post_article, put_article_by_id,
        },
        user::find_user_by_id,
    },
};

#[derive(Serialize, Debug)]
pub struct ArticleResponse {
    articles: Vec<PubArticles>,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct NewArticle {
    pub id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
    pub tags: Option<String>,
}

#[derive(Deserialize, Clone, Serialize, Debug)]
pub struct NewStatus {
    pub toggle: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Params {
    pub id: String,
}

// 获取文章的参数
#[derive(Deserialize, Debug)]
pub struct GetArticlesParams {
    pub condition: Option<String>,
}

/// 获取文章列表
/// 函数签名的参数、返回值都必须能被handle所识别，才能作为路由处理函数
pub async fn articles(
    State(state): State<Arc<AppState>>,
    MaybeJwtAuth(auth): MaybeJwtAuth,
    Query(params): Query<GetArticlesParams>,
) -> AppResult<Json<ArticleResponse>> {
    tracing::info!("Fetching articles with params: {:?}", params);

    let include_unpublished = match auth {
        Some(claims) => is_admin(&state, &claims.user_id).await?,
        None => false,
    };
    let res = get_articles(&state.pool, params, include_unpublished).await?;

    // 显示赋值；
    Ok(Json(ArticleResponse { articles: res }))
}

/// 新增文章
pub async fn handle_post_article(
    State(state): State<Arc<AppState>>,
    JwtAuth(auth): JwtAuth,
    Json(payload): Json<NewArticle>,
) -> AppResult<Json<NewArticle>> {
    // 验证权限
    match find_user_by_id(&state.pool, auth.user_id).await? {
        Some(u) => {
            if u.identity != "admin" {
                return Err(AppError::Forbidden);
            }
        }
        _ => return Err(AppError::BadRequest("No authority".into())),
    };

    let res = post_article(&state.pool, &payload).await?;
    tracing::info!("Posting new article: {:?}", payload.title);

    Ok(Json(res.into()))
}

/// 获取指定文章
pub async fn handle_get_article(
    State(state): State<Arc<AppState>>,
    MaybeJwtAuth(auth): MaybeJwtAuth,
    Path(id): Path<String>, // Path提取器，解析动态路由
) -> AppResult<Json<ArticleModel>> {
    // 不给整个结构体，降低耦合度
    let res = find_article_by_id(&state.pool, &id).await?;

    if let Some(v) = res {
        let can_view_unpublished = match auth {
            Some(claims) => is_admin(&state, &claims.user_id).await?,
            None => false,
        };
        if v.status.as_deref() != Some("published") && !can_view_unpublished {
            return Err(AppError::NotFound);
        }
        tracing::info!("getted article: {:?}", &v.title);
        Ok(Json(v))
    } else {
        Err(AppError::NotFound)
    }
}

/// 删除文章
pub async fn handle_delete_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    JwtAuth(auth): JwtAuth,
) -> AppResult<StatusCode> {
    require_admin(&state, &auth).await?;
    delete_article_by_id(&state.pool, &id).await
}

/// 修改文章
pub async fn handle_put_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    JwtAuth(auth): JwtAuth,
    Json(payload): Json<NewArticle>,
) -> AppResult<Json<ArticleModel>> {
    let res;
    if let Some(v) = payload.id.clone()
        && id != v
    {
        return Err(AppError::BadRequest("Json 与路由信息不一".into()));
    }

    let ident = find_user_by_id(&state.pool, auth.user_id).await?;
    if let Some(v) = ident {
        if v.identity != "admin" {
            return Err(AppError::Forbidden);
        }
        res = put_article_by_id(&state.pool, &id, payload).await?;
        return Ok(Json(res));
    }
    Err(AppError::BadRequest("修改失败".into()))
}

/// 更变文章状态
pub async fn handle_patch_article(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    JwtAuth(auth): JwtAuth,
    Json(payload): Json<NewStatus>,
) -> AppResult<Json<ArticleModel>> {
    let res;

    tracing::info!("Patching article status with payload: {:?}", payload);

    if !matches!(payload.toggle.as_str(), "draft" | "published" | "archived") {
        return Err(AppError::BadRequest("invalid article status".into()));
    }

    if (find_article_by_id(&state.pool, &id).await?).is_none() {
        return Err(AppError::Forbidden);
    }

    let ident = find_user_by_id(&state.pool, auth.user_id).await?;
    if let Some(v) = ident {
        if v.identity != "admin" {
            return Err(AppError::Forbidden);
        }
        res = patch_article_by_id(&state.pool, &id, payload).await?;
        return Ok(Json(res));
    }

    Err(AppError::BadRequest("更变文章状态失败".into()))
}
