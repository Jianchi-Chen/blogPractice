use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use backend::{
    AppState,
    auth::{generate_token, hash_password},
    config::Config,
    create_router,
};
use serde_json::{Value, json};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::{str::FromStr, sync::Arc};
use tower::ServiceExt;

struct TestApp {
    router: Router,
    state: Arc<AppState>,
}

impl TestApp {
    async fn new() -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::from_str("sqlite::memory:")
                    .unwrap()
                    .foreign_keys(true),
            )
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let state = Arc::new(AppState::new(
            pool,
            Config {
                database_url: "sqlite::memory:".into(),
                host: "127.0.0.1".into(),
                port: 0,
                jwt_secret: "test-secret-with-at-least-32-bytes".into(),
                jwt_ttl: 3600,
            },
        ));

        Self {
            router: create_router(state.clone()),
            state,
        }
    }

    async fn insert_user(&self, id: &str, username: &str, identity: &str) -> String {
        let password = hash_password("password").unwrap();
        sqlx::query("INSERT INTO users (id, username, password, identity) VALUES (?, ?, ?, ?)")
            .bind(id)
            .bind(username)
            .bind(password)
            .bind(identity)
            .execute(&self.state.pool)
            .await
            .unwrap();
        generate_token(&self.state, id.to_string(), username).unwrap()
    }

    async fn insert_article(&self, id: &str, title: &str, summary: &str, tags: &str) {
        sqlx::query(
            r#"
            INSERT INTO articles (id, title, content, summary, created_at, status, views, tags)
            VALUES (?, ?, 'content', ?, '2026-01-01T00:00:00Z', 'published', 0, ?)
            "#,
        )
        .bind(id)
        .bind(title)
        .bind(summary)
        .bind(tags)
        .execute(&self.state.pool)
        .await
        .unwrap();
    }

    async fn send(&self, request: Request<Body>) -> (StatusCode, Value) {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        (status, body)
    }

    async fn send_bytes(&self, request: Request<Body>) -> (StatusCode, Vec<u8>, Option<String>) {
        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec();
        (status, bytes, content_type)
    }
}

fn json_request(method: &str, uri: &str, body: Value, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

fn empty_request(method: &str, uri: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(Body::empty()).unwrap()
}

fn avatar_request(uri: &str, token: &str, content_type: &str, bytes: &[u8]) -> Request<Body> {
    let boundary = "myblog-test-boundary";
    let mut body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"avatar\"; filename=\"avatar.png\"\r\nContent-Type: {content_type}\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap()
}

#[tokio::test]
async fn auth_contract_uses_bearer_token_for_current_user() {
    let app = TestApp::new().await;
    let (status, auth) = app
        .send(json_request(
            "POST",
            "/api/auth/register",
            json!({ "username": "reader", "password": "password" }),
            None,
        ))
        .await;
    assert_eq!(status, StatusCode::CREATED);

    let token = auth["token"].as_str().unwrap();
    let (status, current_user) = app
        .send(empty_request("GET", "/api/auth/me", Some(token)))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(current_user["username"], "reader");
    assert_eq!(current_user["identity"], "user");
    assert_eq!(current_user["signature"], "");
    assert!(current_user["avatar_url"].is_null());
}

#[tokio::test]
async fn legacy_web_routes_forward_to_shared_handlers() {
    let app = TestApp::new().await;
    app.insert_user("user-1", "reader", "user").await;
    app.insert_article("article-1", "Vue Guide", "summary", "frontend")
        .await;

    let (login_status, login) = app
        .send(json_request(
            "POST",
            "/api/login",
            json!({ "username": "reader", "password": "password" }),
            None,
        ))
        .await;
    let (articles_status, articles) = app.send(empty_request("GET", "/articles", None)).await;
    let (suggestions_status, suggestions) = app
        .send(empty_request("GET", "/suggestions/vue", None))
        .await;

    assert_eq!(login_status, StatusCode::OK);
    assert!(login["token"].is_string());
    assert_eq!(articles_status, StatusCode::OK);
    assert_eq!(articles["articles"].as_array().unwrap().len(), 1);
    assert_eq!(suggestions_status, StatusCode::OK);
    assert_eq!(suggestions["item"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn unpublished_article_comments_are_only_visible_to_admins() {
    let app = TestApp::new().await;
    let user_token = app.insert_user("user-1", "reader", "user").await;
    let admin_token = app.insert_user("admin-1", "admin-1", "admin").await;
    app.insert_article("draft-1", "Draft", "summary", "tag")
        .await;
    sqlx::query("UPDATE articles SET status = 'draft' WHERE id = 'draft-1'")
        .execute(&app.state.pool)
        .await
        .unwrap();

    let (anonymous_status, _) = app
        .send(empty_request("GET", "/api/articles/draft-1/comments", None))
        .await;
    let (user_status, _) = app
        .send(json_request(
            "POST",
            "/api/articles/draft-1/comments",
            json!({ "content": "hidden" }),
            Some(&user_token),
        ))
        .await;
    let (admin_status, _) = app
        .send(empty_request(
            "GET",
            "/api/articles/draft-1/comments",
            Some(&admin_token),
        ))
        .await;

    assert_eq!(anonymous_status, StatusCode::NOT_FOUND);
    assert_eq!(user_status, StatusCode::NOT_FOUND);
    assert_eq!(admin_status, StatusCode::OK);
}

#[tokio::test]
async fn article_search_and_suggestions_share_case_insensitive_matching() {
    let app = TestApp::new().await;
    app.insert_article("title", "Vue Guide", "summary", "frontend")
        .await;
    app.insert_article("summary", "Rust", "Built with VUE", "backend")
        .await;
    app.insert_article("tags", "Other", "summary", "Vue").await;

    let (status, articles) = app
        .send(empty_request("GET", "/api/articles?query=vue", None))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(articles["articles"].as_array().unwrap().len(), 3);

    let (status, suggestions) = app
        .send(empty_request(
            "GET",
            "/api/search/suggestions?query=vue",
            None,
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(suggestions["items"].as_array().unwrap().len(), 1);
    assert_eq!(suggestions["items"][0]["id"], "title");
}

#[tokio::test]
async fn comment_like_contract_is_idempotent() {
    let app = TestApp::new().await;
    app.insert_article("article-1", "Article", "summary", "tag")
        .await;
    let token = app.insert_user("user-1", "reader", "user").await;

    let (status, comment) = app
        .send(json_request(
            "POST",
            "/api/articles/article-1/comments",
            json!({ "content": "hello", "parent_id": null }),
            Some(&token),
        ))
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let comment_id = comment["comment_id"].as_str().unwrap();

    for expected_count in [1, 1] {
        let (status, like) = app
            .send(json_request(
                "PUT",
                &format!("/api/comments/{comment_id}/like"),
                json!({ "liked": true }),
                Some(&token),
            ))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(like["liked_by_me"], true);
        assert_eq!(like["like_count"], expected_count);
    }

    let (status, unlike) = app
        .send(json_request(
            "PUT",
            &format!("/api/comments/{comment_id}/like"),
            json!({ "liked": false }),
            Some(&token),
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(unlike["liked_by_me"], false);
    assert_eq!(unlike["like_count"], 0);
}

#[tokio::test]
async fn favorites_contract_is_user_scoped_and_idempotent() {
    let app = TestApp::new().await;
    app.insert_article("article-1", "Favorite", "summary", "tag")
        .await;
    let first_token = app.insert_user("user-1", "first", "user").await;
    let second_token = app.insert_user("user-2", "second", "user").await;

    for _ in 0..2 {
        let (status, favorite) = app
            .send(empty_request(
                "PUT",
                "/api/favorites/article-1",
                Some(&first_token),
            ))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(favorite["article_id"], "article-1");
        assert_eq!(favorite["favorited"], true);
    }

    let (_, first_list) = app
        .send(empty_request("GET", "/api/favorites", Some(&first_token)))
        .await;
    let (_, second_list) = app
        .send(empty_request("GET", "/api/favorites", Some(&second_token)))
        .await;
    assert_eq!(first_list["articles"].as_array().unwrap().len(), 1);
    assert_eq!(second_list["articles"].as_array().unwrap().len(), 0);

    for _ in 0..2 {
        let (status, favorite) = app
            .send(empty_request(
                "DELETE",
                "/api/favorites/article-1",
                Some(&first_token),
            ))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(favorite["favorited"], false);
    }
}

#[tokio::test]
async fn profile_signature_contract_persists_on_the_server() {
    let app = TestApp::new().await;
    let token = app.insert_user("user-1", "reader", "user").await;

    let (status, profile) = app
        .send(json_request(
            "PATCH",
            "/api/users/me/profile",
            json!({ "signature": "Shared profile" }),
            Some(&token),
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(profile["signature"], "Shared profile");

    let (_, current_user) = app
        .send(empty_request("GET", "/api/auth/me", Some(&token)))
        .await;
    assert_eq!(current_user["signature"], "Shared profile");
}

#[tokio::test]
async fn protected_contract_endpoints_reject_anonymous_requests() {
    let app = TestApp::new().await;
    for (method, path) in [
        ("GET", "/api/auth/me"),
        ("GET", "/api/favorites"),
        ("PATCH", "/api/users/me/profile"),
    ] {
        let (status, error) = if method == "PATCH" {
            app.send(json_request(method, path, json!({}), None)).await
        } else {
            app.send(empty_request(method, path, None)).await
        };
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(error["code"], 401);
        assert!(error["message"].is_string());
    }
}

#[tokio::test]
async fn login_and_article_admin_lifecycle_follow_the_contract() {
    let app = TestApp::new().await;
    let admin_token = app.insert_user("admin-1", "admin", "admin").await;

    let (status, login) = app
        .send(json_request(
            "POST",
            "/api/auth/login",
            json!({ "username": "admin", "password": "password" }),
            None,
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(login["identity"], "admin");

    let (status, created) = app
        .send(json_request(
            "POST",
            "/api/articles",
            json!({
                "title": "Draft",
                "content": "content",
                "summary": "summary",
                "tags": "tag"
            }),
            Some(&admin_token),
        ))
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["status"], "draft");
    let article_id = created["id"].as_str().unwrap();

    let (status, _) = app
        .send(empty_request(
            "GET",
            &format!("/api/articles/{article_id}"),
            None,
        ))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, updated) = app
        .send(json_request(
            "PUT",
            &format!("/api/articles/{article_id}"),
            json!({
                "id": article_id,
                "title": "Updated",
                "content": "new content",
                "summary": "new summary",
                "tags": "new-tag"
            }),
            Some(&admin_token),
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["title"], "Updated");

    let (status, published) = app
        .send(json_request(
            "PATCH",
            &format!("/api/articles/{article_id}/status"),
            json!({ "status": "published" }),
            Some(&admin_token),
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(published["status"], "published");

    let (status, public_article) = app
        .send(empty_request(
            "GET",
            &format!("/api/articles/{article_id}"),
            None,
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(public_article["title"], "Updated");

    let (status, _) = app
        .send(empty_request(
            "DELETE",
            &format!("/api/articles/{article_id}"),
            Some(&admin_token),
        ))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn user_admin_contract_enforces_permissions_and_updates_accounts() {
    let app = TestApp::new().await;
    let admin_token = app.insert_user("admin-1", "admin", "admin").await;
    let user_token = app.insert_user("user-1", "reader", "user").await;

    let (status, _) = app
        .send(empty_request("GET", "/api/users", Some(&user_token)))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, users) = app
        .send(empty_request(
            "GET",
            "/api/users?limit=20",
            Some(&admin_token),
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(users["users"].as_array().unwrap().len(), 2);

    let (status, _) = app
        .send(json_request(
            "PATCH",
            "/api/users/user-1",
            json!({ "username": "renamed", "identity": "visitor" }),
            Some(&admin_token),
        ))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let row: (String, String) =
        sqlx::query_as("SELECT username, identity FROM users WHERE id = 'user-1'")
            .fetch_one(&app.state.pool)
            .await
            .unwrap();
    assert_eq!(row, ("renamed".into(), "visitor".into()));

    let (status, _) = app
        .send(empty_request(
            "DELETE",
            "/api/users/user-1",
            Some(&admin_token),
        ))
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn comment_listing_reports_the_callers_like_state() {
    let app = TestApp::new().await;
    app.insert_article("article-1", "Article", "summary", "tag")
        .await;
    let token = app.insert_user("user-1", "reader", "user").await;
    let (_, comment) = app
        .send(json_request(
            "POST",
            "/api/articles/article-1/comments",
            json!({ "content": "hello" }),
            Some(&token),
        ))
        .await;
    let comment_id = comment["comment_id"].as_str().unwrap();
    app.send(json_request(
        "PUT",
        &format!("/api/comments/{comment_id}/like"),
        json!({ "liked": true }),
        Some(&token),
    ))
    .await;

    let (_, authenticated) = app
        .send(empty_request(
            "GET",
            "/api/articles/article-1/comments",
            Some(&token),
        ))
        .await;
    let (_, anonymous) = app
        .send(empty_request(
            "GET",
            "/api/articles/article-1/comments",
            None,
        ))
        .await;
    assert_eq!(authenticated["comments"][0]["liked_by_me"], 1);
    assert_eq!(anonymous["comments"][0]["liked_by_me"], 0);
    assert_eq!(anonymous["comments"][0]["like_count"], 1);
}

#[tokio::test]
async fn avatar_contract_validates_persists_serves_and_removes_images() {
    let app = TestApp::new().await;
    let token = app.insert_user("user-1", "reader", "user").await;
    let png = b"\x89PNG\r\n\x1a\ncontract-test";

    let (status, profile) = app
        .send(avatar_request(
            "/api/users/me/avatar",
            &token,
            "image/png",
            png,
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(profile["avatar_url"], "/api/users/user-1/avatar");

    let (status, bytes, content_type) = app
        .send_bytes(empty_request("GET", "/api/users/user-1/avatar", None))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type.as_deref(), Some("image/png"));
    assert_eq!(bytes, png);

    let (status, profile) = app
        .send(empty_request(
            "DELETE",
            "/api/users/me/avatar",
            Some(&token),
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(profile["avatar_url"].is_null());

    let (status, _, _) = app
        .send_bytes(empty_request("GET", "/api/users/user-1/avatar", None))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn invalid_avatar_and_visitor_interactions_are_rejected() {
    let app = TestApp::new().await;
    app.insert_article("article-1", "Article", "summary", "tag")
        .await;
    let visitor_token = app.insert_user("visitor-1", "visitor", "visitor").await;

    let (status, _) = app
        .send(avatar_request(
            "/api/users/me/avatar",
            &visitor_token,
            "image/png",
            b"not-a-png",
        ))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = app
        .send(json_request(
            "POST",
            "/api/articles/article-1/comments",
            json!({ "content": "blocked" }),
            Some(&visitor_token),
        ))
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
