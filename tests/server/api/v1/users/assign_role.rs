use axum::http::StatusCode;
use uuid::Uuid;

use crate::server::api::v1::roles::fixtures::create_role;
use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_http_unauthorized, assert_role_not_found, assert_user_not_found,
};

use super::fixtures::create_user;
use super::requests::assign_role;

#[tokio::test]
async fn should_assign_role_to_user() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role = create_role(&harness, "reader", &[]).await;

    let response = assign_role(&harness, &user.id, &role.id, "admin").await;

    response.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn should_be_idempotent_when_role_is_already_assigned() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role = create_role(&harness, "reader", &[]).await;

    let response = assign_role(&harness, &user.id, &role.id, "admin").await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = assign_role(&harness, &user.id, &role.id, "admin").await;
    response.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::now_v7().to_string();
    let role = create_role(&harness, "reader", &[]).await;

    let response = assign_role(&harness, &user_id, &role.id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_return_not_found_for_unknown_role() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role_id = Uuid::now_v7().to_string();

    let response = assign_role(&harness, &user.id, &role_id, "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_assign_role() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let role = create_role(&harness, "reader", &[]).await;

    let response = harness
        .server
        .put(&format!("/api/v1/users/{}/roles/{}", user.id, role.id))
        .await;

    assert_http_unauthorized(&response);
}
