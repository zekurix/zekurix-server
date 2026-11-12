use axum::http::header;
use axum_test::TestResponse;
use serde_json::Value;

use zekurix_server::authorization::RoleId;

use crate::common::{TestApplication, assert_json_data_error, assert_unauthorized};

use super::common::{
    RoleResponse, assert_duplicate_permission, assert_role_already_exists, assert_role_not_found,
    create_role, get_role,
};

async fn update_role(app: &TestApplication, role_id: RoleId, body: Value) -> TestResponse {
    app.server
        .put(&format!("/api/v1/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&body)
        .await
}

#[tokio::test]
async fn should_update_role_name_and_permissions() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &["roles:read"]).await;

    let response = update_role(
        &app,
        created.id,
        serde_json::json!({
            "name": "editor",
            "permissions": ["roles:read", "roles:update"]
        }),
    )
    .await;

    response.assert_status_ok();

    let updated: RoleResponse = response.json();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "editor");
    assert_eq!(updated.permissions, vec!["roles:read", "roles:update"]);

    let response = get_role(&app, created.id, "admin").await;
    response.assert_status_ok();

    let fetched: RoleResponse = response.json();
    assert_eq!(fetched, updated);
}

#[tokio::test]
async fn should_update_role_without_permissions() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &["roles:read", "roles:update"]).await;

    let response = update_role(
        &app,
        created.id,
        serde_json::json!({
            "name": "reader",
            "permissions": []
        }),
    )
    .await;

    response.assert_status_ok();

    let updated: RoleResponse = response.json();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "reader");
    assert!(updated.permissions.is_empty());

    let response = get_role(&app, created.id, "admin").await;
    response.assert_status_ok();

    let fetched: RoleResponse = response.json();
    assert_eq!(fetched, updated);
}

#[tokio::test]
async fn should_reject_missing_name() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &[]).await;

    let response = update_role(
        &app,
        created.id,
        serde_json::json!({
            "permissions": []
        }),
    )
    .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_missing_permissions() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &[]).await;

    let response = update_role(
        &app,
        created.id,
        serde_json::json!({
            "name": "editor"
        }),
    )
    .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_fields() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &[]).await;

    let response = update_role(
        &app,
        created.id,
        serde_json::json!({
            "name": "editor",
            "permissions": [],
            "unknown_field": 42
        }),
    )
    .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_permissions() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &[]).await;

    let response = update_role(
        &app,
        created.id,
        serde_json::json!({
            "name": "editor",
            "permissions": ["unknown:permission"]
        }),
    )
    .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_duplicate_permissions() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &[]).await;

    let response = update_role(
        &app,
        created.id,
        serde_json::json!({
            "name": "editor",
            "permissions": ["roles:read", "roles:read"]
        }),
    )
    .await;

    assert_duplicate_permission(&response);
}

#[tokio::test]
async fn should_leave_role_unchanged_when_update_fails() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &["roles:read"]).await;

    let response = update_role(
        &app,
        created.id,
        serde_json::json!({
            "name": "editor",
            "permissions": ["unknown:permission"]
        }),
    )
    .await;

    assert_json_data_error(&response);

    let response = get_role(&app, created.id, "admin").await;
    response.assert_status_ok();

    let fetched: RoleResponse = response.json();
    assert_eq!(fetched, created);
}

#[tokio::test]
async fn should_return_conflict_when_updating_to_existing_role_name() {
    let app = TestApplication::new().await;
    let reader = create_role(&app, "reader", &[]).await;
    create_role(&app, "editor", &[]).await;

    let response = update_role(
        &app,
        reader.id,
        serde_json::json!({
            "name": "editor",
            "permissions": []
        }),
    )
    .await;

    assert_role_already_exists(&response);
}

#[tokio::test]
async fn should_return_not_found_when_updating_unknown_role() {
    let app = TestApplication::new().await;

    let response = update_role(
        &app,
        RoleId::new(),
        serde_json::json!({
            "name": "new-role",
            "permissions": []
        }),
    )
    .await;

    assert_role_not_found(&response);
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

    assert_unauthorized(&response);
}
