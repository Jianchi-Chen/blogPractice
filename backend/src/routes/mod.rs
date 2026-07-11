//! HTTP routes shared by the browser and desktop clients.
//! Handlers validate requests, enforce authorization, call repositories, and
//! serialize the canonical API responses.

pub mod articles;
pub mod auth;
pub mod comments;
pub mod favorites;
pub mod health;
mod legacy;
pub mod search;
pub mod users;

// 路由聚合：
// - 将各路由模块组合在一起
// - 添加全局中间件（例如 CORS、Trace）
// - 将全局状态 `AppState` 注入，供提取器与 handler 使用

use crate::db::AppState;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, patch, post, put},
};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

pub fn create_router(state: Arc<AppState>) -> Router {
    // 浏览器跨域请求，一律放行，不拦。
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 路由只负责匹配路径和方法，参数由框架自动提取
    let api = Router::new()
        .route("/health", get(health::health))
        // Canonical HTTP API used by both browser and Tauri.
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::current_user))
        .route("/api/users", get(users::list))
        .route("/api/users/me/profile", patch(users::update_profile))
        .route(
            "/api/users/me/avatar",
            put(users::upload_avatar).delete(users::delete_avatar),
        )
        .route("/api/users/{id}/avatar", get(users::avatar))
        .route(
            "/api/users/{id}",
            patch(users::update).delete(users::delete),
        )
        .route("/api/articles", get(articles::list).post(articles::create))
        .route(
            "/api/articles/{id}",
            get(articles::get)
                .put(articles::update)
                .delete(articles::delete),
        )
        .route("/api/articles/{id}/status", patch(articles::update_status))
        .route(
            "/api/articles/{id}/comments",
            get(comments::list).post(comments::create),
        )
        .route("/api/comments/{id}", delete(comments::delete))
        .route("/api/comments/{id}/like", put(comments::set_like))
        .route("/api/search/suggestions", get(search::suggestions))
        .route("/api/favorites", get(favorites::list))
        .route(
            "/api/favorites/{article_id}",
            put(favorites::add).delete(favorites::remove),
        )
        .merge(legacy::routes())
        .layer(DefaultBodyLimit::max(3 * 1024 * 1024))
        .with_state(state.clone());

    // 返回路由
    Router::new()
        .merge(api)
        .layer(TraceLayer::new_for_http()) // “监控膜”: 每个请求进来/出去时，自动打印日志。
        .layer(cors) // 跨域通行证
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::generate_token;
    use crate::config::Config;
    use crate::db::run_migrations;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use serde_json::{Value, json};
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    async fn test_state() -> Arc<AppState> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        run_migrations(&pool).await.unwrap();
        Arc::new(AppState::new(
            pool,
            Config {
                database_url: "sqlite::memory:".into(),
                host: "127.0.0.1".into(),
                port: 0,
                jwt_secret: "test-secret-with-at-least-32-bytes".into(),
                jwt_ttl: 3600,
            },
        ))
    }

    fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn public_registration_cannot_create_admin() {
        let state = test_state().await;
        let app = create_router(state);
        let response = app
            .oneshot(json_request(
                "POST",
                "/api/auth/register",
                json!({
                    "username": "attacker",
                    "password": "password",
                    "identity": "admin"
                }),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn drafts_are_only_visible_to_admins() {
        let state = test_state().await;
        sqlx::query(
            "INSERT INTO articles (id, title, summary, created_at, status, views, tags) \
             VALUES ('draft-1', 'Draft', '', '2026-01-01', 'draft', 0, '')",
        )
        .execute(&state.pool)
        .await
        .unwrap();
        let app = create_router(state.clone());

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/articles")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(body["articles"].as_array().unwrap().len(), 0);

        let anonymous_detail = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/articles/draft-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(anonymous_detail.status(), StatusCode::NOT_FOUND);

        let token = generate_token(&state, "1".into(), "admin").unwrap();
        let admin_detail = app
            .oneshot(
                Request::builder()
                    .uri("/api/articles/draft-1")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(admin_detail.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn deleting_articles_requires_an_admin() {
        let state = test_state().await;
        sqlx::query(
            "INSERT INTO articles (id, title, summary, created_at, status, views, tags) \
             VALUES ('article-1', 'Article', '', '2026-01-01', 'published', 0, '')",
        )
        .execute(&state.pool)
        .await
        .unwrap();
        let app = create_router(state.clone());

        let anonymous = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/articles/article-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);

        let token = generate_token(&state, "1".into(), "admin").unwrap();
        let admin = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/articles/article-1")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(admin.status(), StatusCode::NO_CONTENT);
    }
}
