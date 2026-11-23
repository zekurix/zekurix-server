use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::server::api::v1::roles::fixtures::create_role;
use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_http_forbidden, assert_http_unauthorized, assert_user_not_found,
};

use super::fixtures::{
    create_user, create_user_with_all_permissions_except, create_user_with_only_permission,
    create_user_without_permissions,
};
use super::requests::{assign_role, list_user_roles};

#[tokio::test]
async fn should_list_roles_assigned_to_user() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let reader = create_role(&harness, "reader", &[]).await;
    let writer = create_role(&harness, "writer", &[]).await;

    assign_role(&harness, &user.id, &reader.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    assign_role(&harness, &user.id, &writer.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = list_user_roles(&harness, &user.id, "admin").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "roles": [
            {
                "id": reader.id,
                "name": reader.name,
            },
            {
                "id": writer.id,
                "name": writer.name,
            }
        ]
    }));
}

#[tokio::test]
async fn should_return_empty_list_when_user_has_no_roles() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let response = list_user_roles(&harness, &user.id, "admin").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "roles": []
    }));
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::now_v7().to_string();

    let response = list_user_roles(&harness, &user_id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_list_user_roles() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let response = harness
        .server
        .get(&format!("/api/v1/users/{}/roles", user.id))
        .await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_list_own_roles_without_roles_read_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let response = list_user_roles(&harness, &user.id, "Alice").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "roles": []
    }));
}

#[tokio::test]
async fn should_list_another_users_roles_for_user_with_only_roles_read_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let reader = create_role(&harness, "reader", &[]).await;
    assign_role(&harness, &user.id, &reader.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);
    let actor = create_user_with_only_permission(&harness, "roles:read").await;

    let response = list_user_roles(&harness, &user.id, &actor).await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "roles": [
            {
                "id": reader.id,
                "name": reader.name,
            }
        ]
    }));
}

#[tokio::test]
async fn should_reject_listing_another_users_roles_without_permissions() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let actor = create_user_without_permissions(&harness).await;

    let response = list_user_roles(&harness, &user.id, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_listing_another_users_roles_without_roles_read_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let actor = create_user_with_all_permissions_except(&harness, "roles:read").await;

    let response = list_user_roles(&harness, &user.id, &actor).await;

    assert_http_forbidden(&response);
}
