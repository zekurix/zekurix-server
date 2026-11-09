use axum::http::{StatusCode, header};
use serde::Deserialize;

use crate::common::ErrorResponse;
use crate::common::TestApplication;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoleResponse {
    id: String,
    name: String,
    permissions: Vec<String>,
}

async fn create_role(app: &TestApplication, name: &str, permissions: &[&str]) -> RoleResponse {
    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": name,
            "permissions": permissions
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    response.json()
}

#[tokio::test]
async fn should_create_role() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": ["roles:read"]
        }))
        .await;

    response.assert_status(StatusCode::CREATED);

    let location = response
        .headers()
        .get(header::LOCATION)
        .expect("Location header should be present")
        .to_str()
        .unwrap()
        .to_owned();

    let role: RoleResponse = response.json();

    assert_eq!(role.name, "user-manager");
    assert_eq!(role.permissions, vec!["roles:read"]);
    assert!(!role.id.is_empty());
    assert_eq!(location, format!("/api/v1/roles/{}", role.id));
}

#[tokio::test]
async fn should_create_role_without_permissions() {
    let app = TestApplication::new().await;

    let role = create_role(&app, "empty-role", &[]).await;

    assert_eq!(role.name, "empty-role");
    assert!(role.permissions.is_empty());
}

#[tokio::test]
async fn should_create_role_with_multiple_permissions() {
    let app = TestApplication::new().await;

    let role = create_role(
        &app,
        "role-with-permissions",
        &["roles:create", "roles:read", "roles:update", "roles:delete"],
    )
    .await;

    assert_eq!(
        role.permissions,
        vec!["roles:create", "roles:read", "roles:update", "roles:delete"]
    );
}

#[tokio::test]
async fn should_reject_missing_name() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "permissions": []
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
async fn should_reject_missing_permissions() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager"
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
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": [],
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
async fn should_reject_unknown_permissions() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": ["unknown:permission"]
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
async fn should_reject_duplicate_permissions() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": ["roles:read", "roles:read"]
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
async fn should_return_conflict_for_existing_role() {
    let app = TestApplication::new().await;

    create_role(&app, "user-manager", &["roles:read"]).await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": ["roles:read"]
        }))
        .await;

    response.assert_status_conflict();
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/role/already-exists"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::CONFLICT.as_u16());
    assert!(!body.detail.is_empty());
}

#[tokio::test]
async fn should_require_authentication_for_create_role() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .json(&serde_json::json!({
            "name": "user-manager",
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
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::UNAUTHORIZED.as_u16());
    assert!(!body.detail.is_empty());
}
