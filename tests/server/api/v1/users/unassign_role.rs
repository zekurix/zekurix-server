use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::server::api::v1::roles::fixtures::create_role;
use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_http_forbidden, assert_http_unauthorized, assert_role_not_found, assert_user_not_found,
};

use super::fixtures::{
    create_user, create_user_with_all_permissions_except, create_user_with_only_permission,
    create_user_without_permissions,
};
use super::requests::{assign_role, list_user_roles, unassign_role};

#[tokio::test]
async fn should_unassign_role_from_user() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role = create_role(&harness, "reader", &[]).await;

    assign_role(&harness, &user.id, &role.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = unassign_role(&harness, &user.id, &role.id, "admin").await;

    response.assert_status(StatusCode::NO_CONTENT);

    let response = list_user_roles(&harness, &user.id, "admin").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "roles": []
    }));
}

#[tokio::test]
async fn should_be_idempotent_when_role_is_not_assigned() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role = create_role(&harness, "reader", &[]).await;

    unassign_role(&harness, &user.id, &role.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    unassign_role(&harness, &user.id, &role.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::now_v7().to_string();
    let role = create_role(&harness, "reader", &[]).await;

    let response = unassign_role(&harness, &user_id, &role.id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_return_not_found_for_unknown_role() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role_id = Uuid::now_v7().to_string();

    let response = unassign_role(&harness, &user.id, &role_id, "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_unassign_role() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role = create_role(&harness, "reader", &[]).await;

    let response = harness
        .server
        .delete(&format!("/api/v1/users/{}/roles/{}", user.id, role.id))
        .await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_unassign_role_for_user_with_only_roles_unassign_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_with_only_permission(&harness, "roles:unassign").await;

    assign_role(&harness, &user.id, &role.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = unassign_role(&harness, &user.id, &role.id, &actor).await;

    response.assert_status(StatusCode::NO_CONTENT);

    let response = list_user_roles(&harness, &user.id, "admin").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "roles": []
    }));
}

#[tokio::test]
async fn should_reject_role_unassignment_for_user_without_permissions() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_without_permissions(&harness).await;

    let response = unassign_role(&harness, &user.id, &role.id, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_role_unassignment_for_user_without_roles_unassign_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role = create_role(&harness, "reader", &[]).await;
    let actor = create_user_with_all_permissions_except(&harness, "roles:unassign").await;

    let response = unassign_role(&harness, &user.id, &role.id, &actor).await;

    assert_http_forbidden(&response);
}
