use crate::server::harness::TestHarness;
use crate::server::problem::assert_http_unauthorized;

use super::fixtures::create_user;
use super::models::UserResponse;
use super::requests::{get_current_user, get_user};

#[tokio::test]
async fn should_get_current_user_after_admin_creation() {
    let harness = TestHarness::new().await;
    let created = create_user(&harness, "Alice").await;

    let response = get_current_user(&harness, "Alice").await;

    response.assert_status_ok();

    let current: UserResponse = response.json();
    assert_eq!(current, created);
}

#[tokio::test]
async fn should_create_current_user_just_in_time() {
    let harness = TestHarness::new().await;

    let response = get_current_user(&harness, "Alice").await;

    response.assert_status_ok();

    let current: UserResponse = response.json();

    let response = get_user(&harness, &current.id, "admin").await;

    response.assert_status_ok();

    let persisted: UserResponse = response.json();
    assert_eq!(persisted, current);
}

#[tokio::test]
async fn should_return_same_user_for_multiple_get_me_requests() {
    let harness = TestHarness::new().await;

    let response = get_current_user(&harness, "Alice").await;
    response.assert_status_ok();

    let first: UserResponse = response.json();

    let response = get_current_user(&harness, "Alice").await;
    response.assert_status_ok();

    let second: UserResponse = response.json();

    assert_eq!(second, first);
}

#[tokio::test]
async fn should_return_different_users_for_different_subjects() {
    let harness = TestHarness::new().await;

    let response = get_current_user(&harness, "Alice").await;
    response.assert_status_ok();

    let alice: UserResponse = response.json();

    let response = get_current_user(&harness, "Bob").await;
    response.assert_status_ok();

    let bob: UserResponse = response.json();

    assert_ne!(alice.id, bob.id);
}

#[tokio::test]
async fn should_require_authentication_for_get_current_user() {
    let harness = TestHarness::new().await;

    let response = harness.server.get("/api/v1/users/me").await;

    assert_http_unauthorized(&response);
}
