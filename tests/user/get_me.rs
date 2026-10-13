use axum::http::{StatusCode, header};

use super::common::UserResponse;
use crate::common::ErrorResponse;
use crate::common::TestApplication;

#[tokio::test]
async fn should_get_current_user_after_admin_creation() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": "https://auth.example.com",
                "subject": "Alice"
            }
        }))
        .await;
    response.assert_status(StatusCode::CREATED);
    let created_user: UserResponse = response.json();

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let current_user: UserResponse = response.json();
    assert_eq!(current_user.id, created_user.id);
}

#[tokio::test]
async fn should_create_current_user_just_in_time() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let current_user: UserResponse = response.json();

    let response = app
        .server
        .get(&format!("/api/v1/users/{}", current_user.id))
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let persisted_user: UserResponse = response.json();
    assert_eq!(persisted_user.id, current_user.id);
}

#[tokio::test]
async fn should_return_same_user_for_multiple_get_me_requests() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let first_user: UserResponse = response.json();

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let second_user: UserResponse = response.json();
    assert_eq!(second_user.id, first_user.id);
}

#[tokio::test]
async fn should_return_different_users_for_different_subjects() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let alice: UserResponse = response.json();

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Bob"))
        .await;
    response.assert_status_ok();
    let bob: UserResponse = response.json();
    assert_ne!(alice.id, bob.id);
}

#[tokio::test]
async fn should_require_authentication_for_get_current_user() {
    let app = TestApplication::new().await;

    let response = app.server.get("/api/v1/users/me").await;
    response.assert_status_unauthorized();
    assert_eq!(response.content_type(), "application/problem+json");
    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/http/unauthorized"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::UNAUTHORIZED.as_u16());
    assert!(!body.detail.is_empty());
}
