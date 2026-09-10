use axum::http::StatusCode;

use zekurix_server::user::UserId;

use super::common::UserResponse;
use crate::common::ErrorResponse;
use crate::common::TestApplication;

#[tokio::test]
async fn should_get_user() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "username": "Alice",
        }))
        .await;
    response.assert_status(StatusCode::CREATED);
    let user_post: UserResponse = response.json();

    let response = app
        .server
        .get(&format!("/api/v1/users/{}", user_post.id))
        .await;
    response.assert_status_ok();
    let user_get: UserResponse = response.json();
    assert_eq!(user_get.id, user_post.id);
    assert_eq!(user_get.username, "Alice");
}

#[tokio::test]
async fn should_create_and_get_multiple_users() {
    let app = TestApplication::new().await;
    let usernames = ["Alice", "Bob", "Charlie"];

    for username in usernames {
        let response = app
            .server
            .post("/api/v1/users")
            .json(&serde_json::json!({
                "username": username,
            }))
            .await;
        response.assert_status(StatusCode::CREATED);
        let user_post: UserResponse = response.json();

        let response = app
            .server
            .get(&format!("/api/v1/users/{}", user_post.id))
            .await;
        response.assert_status_ok();
        let user_get: UserResponse = response.json();
        assert_eq!(user_get.id, user_post.id);
        assert_eq!(user_get.username, username);
    }
}

#[tokio::test]
async fn should_return_not_found_for_invalid_user_id() {
    let app = TestApplication::new().await;

    let user_id = UserId::new();
    let response = app.server.get(&format!("/api/v1/users/{}", user_id)).await;
    response.assert_status_not_found();

    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/user/not-found"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::NOT_FOUND.as_u16());
    assert!(!body.detail.is_empty());
}
