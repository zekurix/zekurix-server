use axum::{body::Bytes, http::StatusCode};

use crate::common::ErrorResponse;
use crate::common::TestApplication;

#[tokio::test]
async fn should_return_problem_details_for_json_data_error() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!(42))
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
async fn should_return_problem_details_for_json_syntax_error() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .bytes(Bytes::from("not a json"))
        .add_header("content-type", "application/json")
        .await;
    response.assert_status_bad_request();
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/json/syntax-error"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::BAD_REQUEST.as_u16());
    assert!(!body.detail.is_empty());
}

#[tokio::test]
async fn should_return_problem_details_for_missing_json_content_type() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .bytes(Bytes::from(r#"{"username":"Alice"}"#))
        .await;
    response.assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/json/missing-content-type"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::UNSUPPORTED_MEDIA_TYPE.as_u16());
    assert!(!body.detail.is_empty());
}

#[tokio::test]
async fn should_return_problem_details_for_json_bytes_rejection() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .bytes(Bytes::from(vec![b'a'; 3_000_000]))
        .add_header("content-type", "application/json")
        .await;
    response.assert_status(StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/json/bytes-rejection"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::PAYLOAD_TOO_LARGE.as_u16());
    assert!(!body.detail.is_empty());
}
