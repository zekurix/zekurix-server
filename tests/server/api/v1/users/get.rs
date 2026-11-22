use uuid::Uuid;

use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_http_forbidden, assert_http_unauthorized, assert_user_not_found,
};

use super::fixtures::{
    create_user, create_user_with_all_permissions_except, create_user_with_only_permission,
    create_user_without_permissions,
};
use super::models::UserResponse;
use super::requests::get_user;

#[tokio::test]
async fn should_get_user() {
    let harness = TestHarness::new().await;
    let created = create_user(&harness, "Alice").await;

    let response = get_user(&harness, &created.id, "admin").await;

    response.assert_status_ok();

    let fetched: UserResponse = response.json();
    assert_eq!(fetched, created);
}

#[tokio::test]
async fn should_create_and_get_multiple_users() {
    let harness = TestHarness::new().await;

    for subject in ["Alice", "Bob", "Charlie"] {
        let created = create_user(&harness, subject).await;

        let response = get_user(&harness, &created.id, "admin").await;

        response.assert_status_ok();

        let fetched: UserResponse = response.json();
        assert_eq!(fetched, created);
    }
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::now_v7().to_string();

    let response = get_user(&harness, &user_id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_get_user() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;

    let response = harness
        .server
        .get(&format!("/api/v1/users/{}", user.id))
        .await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_get_user_for_user_with_only_users_read_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let actor = create_user_with_only_permission(&harness, "users:read").await;

    let response = get_user(&harness, &user.id, &actor).await;

    response.assert_status_ok();
    let fetched: UserResponse = response.json();
    assert_eq!(fetched, user);
}

#[tokio::test]
async fn should_reject_get_user_for_user_without_permissions() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let actor = create_user_without_permissions(&harness).await;

    let response = get_user(&harness, &user.id, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_get_user_for_user_without_users_read_permission() {
    let harness = TestHarness::new().await;
    let user = create_user(&harness, "Alice").await;
    let actor = create_user_with_all_permissions_except(&harness, "users:read").await;

    let response = get_user(&harness, &user.id, &actor).await;

    assert_http_forbidden(&response);
}
