use axum::http::StatusCode;

use crate::common::ErrorResponse;
use crate::common::TestApplication;

#[tokio::test]
async fn should_return_not_found_for_unknown_route() {
    let app = TestApplication::new().await;

    let response = app.server.get("/foobar").await;
    response.assert_status_not_found();

    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/http/not-found"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::NOT_FOUND.as_u16());
    assert!(!body.detail.is_empty());
}

#[tokio::test]
async fn should_return_method_not_allowed_for_invalid_method() {
    let app = TestApplication::new().await;

    let response = app.server.post("/health").await;
    assert_eq!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);

    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/http/method-not-allowed"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::METHOD_NOT_ALLOWED.as_u16());
    assert!(!body.detail.is_empty());
}
