use axum::http::{StatusCode, header};

use zekurix_server::authorization::RoleId;

use super::common::{assert_role_not_found, create_role};
use crate::common::{ErrorResponse, TestApplication};

#[tokio::test]
async fn should_delete_role() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &["roles:read"]).await;

    let response = app
        .server
        .delete(&format!("/api/v1/roles/{}", role.id))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = app
        .server
        .get(&format!("/api/v1/roles/{}", role.id))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;
    assert_role_not_found(response);
}

#[tokio::test]
async fn should_return_not_found_when_deleting_unknown_role() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .delete(&format!("/api/v1/roles/{}", RoleId::new()))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;

    assert_role_not_found(response);
}

#[tokio::test]
async fn should_require_authentication_to_delete_role() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;

    let response = app
        .server
        .delete(&format!("/api/v1/roles/{}", role.id))
        .await;

    response.assert_status_unauthorized();
    assert_eq!(response.content_type(), "application/problem+json");
    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/http/unauthorized"
    );
    assert_eq!(body.status, StatusCode::UNAUTHORIZED.as_u16());
    assert!(!body.title.is_empty());
    assert!(!body.detail.is_empty());
}
