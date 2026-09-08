use axum::http::StatusCode;

use crate::common::ErrorResponse;
use crate::common::TestApplication;

#[tokio::test]
async fn should_return_problem_details_for_path_rejection() {
    let app = TestApplication::new().await;

    let response = app.server.get("/api/v1/users/invalid-uuid").await;
    response.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/path/rejection"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::BAD_REQUEST.as_u16());
    assert!(!body.detail.is_empty());
}
