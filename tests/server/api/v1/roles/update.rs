use serde_json::json;
use uuid::Uuid;

use crate::server::api::v1::users::fixtures::{
    create_user_with_all_permissions_except, create_user_with_only_permission,
    create_user_without_permissions,
};
use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_http_forbidden, assert_http_unauthorized, assert_json_data_error,
    assert_permission_duplicate, assert_role_already_exists, assert_role_not_found,
};

use super::fixtures::create_role;
use super::models::RoleResponse;
use super::requests::{get_role, update_role};

#[tokio::test]
async fn should_update_role_name_and_permissions() {
    let harness = TestHarness::new().await;
    let created = create_role(&harness, "reader", &["roles:read"]).await;

    let body = json!({
        "name": "editor",
        "permissions": ["roles:read", "roles:update"],
    });

    let response = update_role(&harness, &created.id, body, "admin").await;

    response.assert_status_ok();

    let updated: RoleResponse = response.json();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "editor");
    assert_eq!(updated.permissions, vec!["roles:read", "roles:update"]);

    let response = get_role(&harness, &created.id, "admin").await;
    response.assert_status_ok();

    let fetched: RoleResponse = response.json();
    assert_eq!(fetched, updated);
}

#[tokio::test]
async fn should_update_role_without_permissions() {
    let harness = TestHarness::new().await;
    let created = create_role(&harness, "reader", &["roles:read", "roles:update"]).await;

    let body = json!({
        "name": "reader",
        "permissions": [],
    });

    let response = update_role(&harness, &created.id, body, "admin").await;

    response.assert_status_ok();

    let updated: RoleResponse = response.json();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "reader");
    assert!(updated.permissions.is_empty());

    let response = get_role(&harness, &created.id, "admin").await;
    response.assert_status_ok();

    let fetched: RoleResponse = response.json();
    assert_eq!(fetched, updated);
}

#[tokio::test]
async fn should_reject_missing_name() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;

    let body = json!({
        "permissions": [],
    });

    let response = update_role(&harness, &role.id, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_missing_permissions() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;

    let body = json!({
        "name": "editor",
    });

    let response = update_role(&harness, &role.id, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_fields() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;

    let body = json!({
        "name": "editor",
        "permissions": [],
        "unknown_field": 42,
    });

    let response = update_role(&harness, &role.id, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_permissions() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;

    let body = json!({
        "name": "editor",
        "permissions": ["unknown:permission"],
    });

    let response = update_role(&harness, &role.id, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_duplicate_permissions() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;

    let body = json!({
        "name": "editor",
        "permissions": ["roles:read", "roles:read"],
    });

    let response = update_role(&harness, &role.id, body, "admin").await;

    assert_permission_duplicate(&response);
}

#[tokio::test]
async fn should_leave_role_unchanged_when_update_fails() {
    let harness = TestHarness::new().await;
    let created = create_role(&harness, "reader", &["roles:read"]).await;

    let body = json!({
        "name": "editor",
        "permissions": ["unknown:permission"],
    });

    let response = update_role(&harness, &created.id, body, "admin").await;

    assert_json_data_error(&response);

    let response = get_role(&harness, &created.id, "admin").await;
    response.assert_status_ok();

    let fetched: RoleResponse = response.json();
    assert_eq!(fetched, created);
}

#[tokio::test]
async fn should_return_conflict_when_updating_to_existing_role_name() {
    let harness = TestHarness::new().await;
    let reader = create_role(&harness, "reader", &[]).await;

    create_role(&harness, "editor", &[]).await;

    let body = json!({
        "name": "editor",
        "permissions": [],
    });

    let response = update_role(&harness, &reader.id, body, "admin").await;

    assert_role_already_exists(&response);
}

#[tokio::test]
async fn should_return_not_found_when_updating_unknown_role() {
    let harness = TestHarness::new().await;
    let role_id = Uuid::now_v7().to_string();

    let body = json!({
        "name": "new-role",
        "permissions": [],
    });

    let response = update_role(&harness, &role_id, body, "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_update_role() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;

    let body = json!({
        "name": "new-role",
        "permissions": [],
    });

    let response = harness
        .server
        .put(&format!("/api/v1/roles/{}", role.id))
        .json(&body)
        .await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_update_role_for_user_with_only_roles_update_permission() {
    let harness = TestHarness::new().await;
    let created = create_role(&harness, "reader", &[]).await;
    let actor = create_user_with_only_permission(&harness, "roles:update").await;

    let body = json!({
        "name": "editor",
        "permissions": ["roles:read"],
    });

    let response = update_role(&harness, &created.id, body, &actor).await;

    response.assert_status_ok();

    let updated: RoleResponse = response.json();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "editor");
    assert_eq!(updated.permissions, vec!["roles:read"]);
}

#[tokio::test]
async fn should_reject_role_update_for_user_without_permissions() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_without_permissions(&harness).await;

    let body = json!({
        "name": "editor",
        "permissions": [],
    });

    let response = update_role(&harness, &role.id, body, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_role_update_for_user_without_roles_update_permission() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_with_all_permissions_except(&harness, "roles:update").await;

    let body = json!({
        "name": "editor",
        "permissions": [],
    });

    let response = update_role(&harness, &role.id, body, &actor).await;

    assert_http_forbidden(&response);
}
