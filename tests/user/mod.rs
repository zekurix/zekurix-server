use axum::http::StatusCode;
use serde::Deserialize;
use test_case::test_case;

use zekurix_server::user::UserId;

use crate::common::TestApplication;

#[derive(Deserialize)]
struct UserResponse {
    id: UserId,
    username: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    code: String,
}

#[tokio::test]
async fn should_create_user() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/users")
        .json(&serde_json::json!({
            "username": "Alice",
        }))
        .await;
    response.assert_status(StatusCode::CREATED);

    let user: UserResponse = response.json();
    assert_eq!(user.username, "Alice");
}

#[test_case("" ; "empty username")]
#[test_case("A" ; "one character")]
#[test_case("AB" ; "two characters")]
#[test_case("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789012" ; "too long")]
#[test_case("Alice!" ; "invalid characters")]
#[tokio::test]
async fn should_reject_invalid_user(username: &str) {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/users")
        .json(&serde_json::json!({
            "username": username,
        }))
        .await;

    response.assert_status_unprocessable_entity();
}

#[tokio::test]
async fn should_get_user() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/users")
        .json(&serde_json::json!({
            "username": "Alice",
        }))
        .await;
    response.assert_status(StatusCode::CREATED);
    let user_post: UserResponse = response.json();

    let response = app.server.get(&format!("/users/{}", user_post.id)).await;
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
            .post("/users")
            .json(&serde_json::json!({
                "username": username,
            }))
            .await;
        response.assert_status(StatusCode::CREATED);
        let user_post: UserResponse = response.json();

        let response = app.server.get(&format!("/users/{}", user_post.id)).await;
        response.assert_status_ok();
        let user_get: UserResponse = response.json();
        assert_eq!(user_get.id, user_post.id);
        assert_eq!(user_get.username, username);
    }
}

#[tokio::test]
async fn should_return_conflict_for_existing_user() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/users")
        .json(&serde_json::json!({
            "username": "Alice",
        }))
        .await;
    response.assert_status(StatusCode::CREATED);

    let response = app
        .server
        .post("/users")
        .json(&serde_json::json!({
            "username": "Alice",
        }))
        .await;
    response.assert_status_conflict();
    let error: ErrorResponse = response.json();
    assert_eq!(error.code, "USER_ALREADY_EXISTS");
}

#[tokio::test]
async fn should_return_not_found_for_invalid_user_id() {
    let app = TestApplication::new().await;

    let user_id = UserId::new();
    let response = app.server.get(&format!("/users/{}", user_id)).await;
    response.assert_status_not_found();

    let error: ErrorResponse = response.json();
    assert_eq!(error.code, "USER_NOT_FOUND");
}

#[tokio::test]
async fn should_return_unprocessable_entity_for_invalid_payload() {
    let app = TestApplication::new().await;

    let response = app.server.post("/users").json(&serde_json::json!({})).await;
    response.assert_status_unprocessable_entity();
}
