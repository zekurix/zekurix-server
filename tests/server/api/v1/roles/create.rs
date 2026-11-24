use axum::http::{StatusCode, header};
use serde_json::json;
use uuid::Uuid;

use crate::server::api::v1::users::fixtures::{
    create_user_with_all_permissions_except, create_user_with_only_permission,
    create_user_without_permissions,
};
use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_http_forbidden, assert_http_unauthorized, assert_json_data_error,
    assert_permission_duplicate, assert_role_already_exists,
};

use super::fixtures::create_role;
use super::models::RoleResponse;
use super::requests::create_role as request_create_role;

#[tokio::test]
async fn should_create_role() {
    let harness = TestHarness::new().await;

    let body = json!({
        "name": "user-manager",
        "permissions": ["roles:read"],
    });

    let response = request_create_role(&harness, body, "admin").await;

    response.assert_status(StatusCode::CREATED);

    let location = response
        .headers()
        .get(header::LOCATION)
        .expect("Location header should be present")
        .to_str()
        .unwrap();

    let role: RoleResponse = response.json();

    Uuid::parse_str(&role.id).expect("role ID should be a valid UUID");

    assert_eq!(role.name, "user-manager");
    assert_eq!(role.permissions, vec!["roles:read"]);
    assert_eq!(location, format!("/api/v1/roles/{}", role.id));
}

#[tokio::test]
async fn should_create_role_without_permissions() {
    let harness = TestHarness::new().await;

    let role = create_role(&harness, "empty-role", &[]).await;

    assert_eq!(role.name, "empty-role");
    assert!(role.permissions.is_empty());
}

#[tokio::test]
async fn should_create_role_with_multiple_permissions() {
    let harness = TestHarness::new().await;

    let role = create_role(
        &harness,
        "role-with-permissions",
        &["roles:create", "roles:read", "roles:update", "roles:delete"],
    )
    .await;

    assert_eq!(
        role.permissions,
        vec!["roles:create", "roles:read", "roles:update", "roles:delete",]
    );
}

#[tokio::test]
async fn should_reject_missing_name() {
    let harness = TestHarness::new().await;

    let body = json!({
        "permissions": [],
    });

    let response = request_create_role(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_missing_permissions() {
    let harness = TestHarness::new().await;

    let body = json!({
        "name": "user-manager",
    });

    let response = request_create_role(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_fields() {
    let harness = TestHarness::new().await;

    let body = json!({
        "name": "user-manager",
        "permissions": [],
        "unknown_field": 42,
    });

    let response = request_create_role(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_permissions() {
    let harness = TestHarness::new().await;

    let body = json!({
        "name": "user-manager",
        "permissions": ["unknown:permission"],
    });

    let response = request_create_role(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_duplicate_permissions() {
    let harness = TestHarness::new().await;

    let body = json!({
        "name": "user-manager",
        "permissions": ["roles:read", "roles:read"],
    });

    let response = request_create_role(&harness, body, "admin").await;

    assert_permission_duplicate(&response);
}

#[tokio::test]
async fn should_return_conflict_for_existing_role() {
    let harness = TestHarness::new().await;

    create_role(&harness, "user-manager", &["roles:read"]).await;

    let body = json!({
        "name": "user-manager",
        "permissions": ["roles:read"],
    });

    let response = request_create_role(&harness, body, "admin").await;

    assert_role_already_exists(&response);
}

#[tokio::test]
async fn should_require_authentication_for_create_role() {
    let harness = TestHarness::new().await;

    let body = json!({
        "name": "user-manager",
        "permissions": [],
    });

    let response = harness.server.post("/api/v1/roles").json(&body).await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_create_role_for_user_with_only_roles_create_permission() {
    let harness = TestHarness::new().await;
    let actor = create_user_with_only_permission(&harness, "roles:create").await;

    let body = json!({
        "name": "user-manager",
        "permissions": [],
    });

    let response = request_create_role(&harness, body, &actor).await;

    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn should_reject_role_creation_with_permissions_the_user_does_not_have() {
    let harness = TestHarness::new().await;
    let actor = create_user_with_only_permission(&harness, "roles:create").await;

    let body = json!({
        "name": "privileged-role",
        "permissions": ["roles:delete"],
    });

    let response = request_create_role(&harness, body, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_role_creation_for_user_without_permissions() {
    let harness = TestHarness::new().await;
    let actor = create_user_without_permissions(&harness).await;

    let body = json!({
        "name": "user-manager",
        "permissions": [],
    });

    let response = request_create_role(&harness, body, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_role_creation_for_user_without_roles_create_permission() {
    let harness = TestHarness::new().await;
    let actor = create_user_with_all_permissions_except(&harness, "roles:create").await;

    let body = json!({
        "name": "user-manager",
        "permissions": [],
    });

    let response = request_create_role(&harness, body, &actor).await;

    assert_http_forbidden(&response);
}
