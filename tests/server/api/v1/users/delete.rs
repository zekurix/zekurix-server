use axum::http::StatusCode;
use uuid::Uuid;

use crate::server::harness::TestHarness;
use crate::server::problem::{assert_http_unauthorized, assert_user_not_found};

use super::fixtures::create_user;
use super::models::UserResponse;
use super::requests::{delete_user, get_current_user, get_user};

#[tokio::test]
async fn should_delete_user_as_admin() {
    let harness = TestHarness::new().await;
    let alice = create_user(&harness, "Alice").await;

    let response = delete_user(&harness, &alice.id, "admin").await;

    response.assert_status(StatusCode::NO_CONTENT);

    let response = get_user(&harness, &alice.id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_delete_current_user() {
    let harness = TestHarness::new().await;
    let alice = create_user(&harness, "Alice").await;

    let response = delete_user(&harness, &alice.id, "Alice").await;

    response.assert_status(StatusCode::NO_CONTENT);

    let response = get_user(&harness, &alice.id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_recreate_user_after_login_following_deletion() {
    let harness = TestHarness::new().await;
    let alice = create_user(&harness, "Alice").await;

    let response = delete_user(&harness, &alice.id, "admin").await;

    response.assert_status(StatusCode::NO_CONTENT);

    let response = get_current_user(&harness, "Alice").await;

    response.assert_status_ok();

    let recreated: UserResponse = response.json();

    assert_ne!(recreated.id, alice.id);

    let response = get_user(&harness, &recreated.id, "admin").await;

    response.assert_status_ok();
}

#[tokio::test]
async fn should_return_not_found_when_deleting_unknown_user() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::now_v7().to_string();

    let response = delete_user(&harness, &user_id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_return_not_found_when_deleting_user_twice() {
    let harness = TestHarness::new().await;
    let alice = create_user(&harness, "Alice").await;

    let response = delete_user(&harness, &alice.id, "admin").await;

    response.assert_status(StatusCode::NO_CONTENT);

    let response = delete_user(&harness, &alice.id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_delete_user() {
    let harness = TestHarness::new().await;
    let alice = create_user(&harness, "Alice").await;

    let response = harness
        .server
        .delete(&format!("/api/v1/users/{}", alice.id))
        .await;

    assert_http_unauthorized(&response);
}
