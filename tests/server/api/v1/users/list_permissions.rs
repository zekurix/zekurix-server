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
use super::requests::{assign_role, list_user_permissions};

#[tokio::test]
async fn should_list_permissions_granted_to_user() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let reader = create_role(&harness, "reader", &["roles:read"]).await;
    let editor = create_role(&harness, "editor", &["roles:update"]).await;

    assign_role(&harness, &user.id, &reader.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    assign_role(&harness, &user.id, &editor.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = list_user_permissions(&harness, &user.id, "admin").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "permissions": [
            "roles:read",
            "roles:update",
        ]
    }));
}

#[tokio::test]
async fn should_list_distinct_permissions_from_multiple_roles() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let reader = create_role(&harness, "reader", &["roles:read", "roles:update"]).await;

    let admin = create_role(&harness, "admin-role", &["roles:update", "roles:delete"]).await;

    assign_role(&harness, &user.id, &reader.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    assign_role(&harness, &user.id, &admin.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = list_user_permissions(&harness, &user.id, "admin").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "permissions": [
            "roles:delete",
            "roles:read",
            "roles:update",
        ]
    }));
}

#[tokio::test]
async fn should_return_empty_list_when_user_has_no_permissions() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let response = list_user_permissions(&harness, &user.id, "admin").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "permissions": []
    }));
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::now_v7().to_string();

    let response = list_user_permissions(&harness, &user_id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_list_user_permissions() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let response = harness
        .server
        .get(&format!("/api/v1/users/{}/permissions", user.id))
        .await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_list_own_permissions_without_roles_read_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let response = list_user_permissions(&harness, &user.id, "Alice").await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "permissions": []
    }));
}

#[tokio::test]
async fn should_list_another_users_permissions_for_user_with_only_roles_read_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let actor = create_user_with_only_permission(&harness, "roles:read").await;

    let response = list_user_permissions(&harness, &user.id, &actor).await;

    response.assert_status_ok();
}

#[tokio::test]
async fn should_reject_listing_another_users_permissions_without_permissions() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let actor = create_user_without_permissions(&harness).await;

    let response = list_user_permissions(&harness, &user.id, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_listing_another_users_permissions_without_roles_read_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let actor = create_user_with_all_permissions_except(&harness, "roles:read").await;

    let response = list_user_permissions(&harness, &user.id, &actor).await;

    assert_http_forbidden(&response);
}
