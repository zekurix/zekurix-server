use axum::http::{StatusCode, header};

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
            "identity": {
                "issuer": "https://auth.example.com",
                "subject": "Alice"
            }
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
}

#[tokio::test]
async fn should_reject_missing_identity() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({}))
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
async fn should_reject_missing_identity_issuer() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "identity": {
                "subject": "Alice"
            }
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
async fn should_reject_missing_identity_subject() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "identity": {
                "issuer": "https://auth.example.com",
            }
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
async fn should_reject_empty_identity_issuer() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "identity": {
                "issuer": "",
                "subject": "Alice"
            }
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
async fn should_reject_empty_identity_subject() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "identity": {
                "issuer": "https://auth.example.com",
                "subject": ""
            }
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
            "identity": {
                "issuer": "https://auth.example.com",
                "subject": "Alice"
            },
            "unknown_field": 42
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
async fn should_reject_unknown_identity_fields() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "identity": {
                "issuer": "https://auth.example.com",
                "subject": "Alice",
                "unknown_field": 42
            }
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
            "identity": {
                "issuer": "https://auth.example.com",
                "subject": "Alice"
            }
        }))
        .await;
    response.assert_status(StatusCode::CREATED);

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "identity": {
                "issuer": "https://auth.example.com",
                "subject": "Alice"
            }
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
