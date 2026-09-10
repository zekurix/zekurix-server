use axum::http::{StatusCode, header};
use test_case::test_case;

use super::common::UserResponse;
use crate::common::ErrorResponse;
use crate::common::TestApplication;

#[tokio::test]
async fn should_create_user() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "username": "Alice",
        }))
        .await;
    response.assert_status(StatusCode::CREATED);

    let location = response
        .headers()
        .get(header::LOCATION)
        .expect("Location header should be present")
        .to_str()
        .unwrap();
    let user: UserResponse = response.json();

    assert_eq!(location, format!("/api/v1/users/{}", user.id));
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
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "username": username,
        }))
        .await;

    response.assert_status_unprocessable_entity();

    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/json/data-error"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::UNPROCESSABLE_ENTITY.as_u16());
    assert!(!body.detail.is_empty());
}

#[tokio::test]
async fn should_reject_unknown_fields() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
        "username": "Alice",
        "unknown_field": 42,
        }))
        .await;

    response.assert_status_unprocessable_entity();

    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/json/data-error"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::UNPROCESSABLE_ENTITY.as_u16());
    assert!(!body.detail.is_empty());
}

#[tokio::test]
async fn should_return_conflict_for_existing_user() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "username": "Alice",
        }))
        .await;
    response.assert_status(StatusCode::CREATED);

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "username": "Alice",
        }))
        .await;
    response.assert_status_conflict();

    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/user/already-exists"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::CONFLICT.as_u16());
    assert!(!body.detail.is_empty());
}

#[tokio::test]
async fn should_return_unprocessable_entity_for_invalid_payload() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({}))
        .await;
    response.assert_status_unprocessable_entity();
}
