use uuid::Uuid;

use crate::server::api::v1::users::fixtures::{
    create_user_with_all_permissions_except, create_user_with_only_permission,
    create_user_without_permissions,
};
use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_http_forbidden, assert_http_unauthorized, assert_role_not_found,
};

use super::fixtures::create_role;
use super::models::RoleResponse;
use super::requests::get_role;

#[tokio::test]
async fn should_get_role() {
    let harness = TestHarness::new().await;
    let created = create_role(&harness, "reader", &["roles:read"]).await;

    let response = get_role(&harness, &created.id, "admin").await;

    response.assert_status_ok();
    assert_eq!(response.content_type(), "application/json");

    let role: RoleResponse = response.json();
    assert_eq!(role, created);
}

#[tokio::test]
async fn should_get_multiple_roles() {
    let harness = TestHarness::new().await;

    let reader = create_role(&harness, "reader", &["roles:read"]).await;
    let writer = create_role(&harness, "writer", &["roles:update"]).await;
    let admin = create_role(&harness, "admin", &["roles:read", "roles:update"]).await;

    for expected in [reader, writer, admin] {
        let response = get_role(&harness, &expected.id, "admin").await;

        response.assert_status_ok();
        assert_eq!(response.content_type(), "application/json");

        let role: RoleResponse = response.json();
        assert_eq!(role, expected);
    }
}

#[tokio::test]
async fn should_return_not_found_for_unknown_role() {
    let harness = TestHarness::new().await;
    let role_id = Uuid::now_v7().to_string();

    let response = get_role(&harness, &role_id, "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_get_role() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;

    let response = harness
        .server
        .get(&format!("/api/v1/roles/{}", role.id))
        .await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_get_role_for_user_with_only_roles_read_permission() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_with_only_permission(&harness, "roles:read").await;

    let response = get_role(&harness, &role.id, &actor).await;

    response.assert_status_ok();

    let fetched: RoleResponse = response.json();
    assert_eq!(fetched, role);
}

#[tokio::test]
async fn should_reject_get_role_for_user_without_permissions() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_without_permissions(&harness).await;

    let response = get_role(&harness, &role.id, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_get_role_for_user_without_roles_read_permission() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_with_all_permissions_except(&harness, "roles:read").await;

    let response = get_role(&harness, &role.id, &actor).await;

    assert_http_forbidden(&response);
}
