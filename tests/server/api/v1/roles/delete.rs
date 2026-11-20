use axum::http::StatusCode;
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
use super::requests::{delete_role, get_role};

#[tokio::test]
async fn should_delete_role() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &["roles:read"]).await;

    let response = delete_role(&harness, &role.id, "admin").await;

    response.assert_status(StatusCode::NO_CONTENT);

    let response = get_role(&harness, &role.id, "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_return_not_found_when_deleting_role_twice() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &["roles:read"]).await;

    let response = delete_role(&harness, &role.id, "admin").await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = delete_role(&harness, &role.id, "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_return_not_found_when_deleting_unknown_role() {
    let harness = TestHarness::new().await;
    let role_id = Uuid::now_v7().to_string();

    let response = delete_role(&harness, &role_id, "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_delete_role() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;

    let response = harness
        .server
        .delete(&format!("/api/v1/roles/{}", role.id))
        .await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_delete_role_for_user_with_only_roles_delete_permission() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_with_only_permission(&harness, "roles:delete").await;

    let response = delete_role(&harness, &role.id, &actor).await;

    response.assert_status(StatusCode::NO_CONTENT);

    let response = get_role(&harness, &role.id, "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_reject_role_deletion_for_user_without_permissions() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_without_permissions(&harness).await;

    let response = delete_role(&harness, &role.id, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_role_deletion_for_user_without_roles_delete_permission() {
    let harness = TestHarness::new().await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_with_all_permissions_except(&harness, "roles:delete").await;

    let response = delete_role(&harness, &role.id, &actor).await;

    assert_http_forbidden(&response);
}
