use axum::http::{StatusCode, header};

use zekurix_server::authorization::RoleId;

use super::common::{RoleResponse, assert_role_not_found, create_role};
use crate::common::{ErrorResponse, TestApplication};

#[tokio::test]
async fn should_update_role_name_and_permissions() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &["roles:read"]).await;

    let response = app
        .server
        .put(&format!("/api/v1/roles/{}", created.id))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "editor",
            "permissions": ["roles:read", "roles:update"]
        }))
        .await;

    response.assert_status_ok();
    let updated: RoleResponse = response.json();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "editor");
    assert_eq!(updated.permissions, vec!["roles:read", "roles:update"]);

    let response = app
        .server
        .get(&format!("/api/v1/roles/{}", created.id))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;
    response.assert_status_ok();
    let fetched: RoleResponse = response.json();
    assert_eq!(fetched, updated);
}

#[tokio::test]
async fn should_return_not_found_when_updating_unknown_role() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .put(&format!("/api/v1/roles/{}", RoleId::new()))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "new-role",
            "permissions": []
        }))
        .await;

    assert_role_not_found(response);
}

#[tokio::test]
async fn should_require_authentication_to_update_role() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;

    let response = app
        .server
        .put(&format!("/api/v1/roles/{}", role.id))
        .json(&serde_json::json!({
            "name": "editor",
            "permissions": []
        }))
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
