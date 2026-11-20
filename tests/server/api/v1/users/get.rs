use uuid::Uuid;

use crate::server::harness::TestHarness;
use crate::server::problem::{assert_http_unauthorized, assert_user_not_found};

use super::fixtures::create_user;
use super::models::UserResponse;
use super::requests::get_user;

#[tokio::test]
async fn should_get_user() {
    let harness = TestHarness::new().await;
    let created = create_user(&harness, "Alice").await;

    let response = get_user(&harness, &created.id, "Alice").await;

    response.assert_status_ok();

    let fetched: UserResponse = response.json();
    assert_eq!(fetched, created);
}

#[tokio::test]
async fn should_create_and_get_multiple_users() {
    let harness = TestHarness::new().await;

    for subject in ["Alice", "Bob", "Charlie"] {
        let created = create_user(&harness, subject).await;

        let response = get_user(&harness, &created.id, subject).await;

        response.assert_status_ok();

        let fetched: UserResponse = response.json();
        assert_eq!(fetched, created);
    }
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    let harness = TestHarness::new().await;
    let user_id = Uuid::now_v7().to_string();

    let response = get_user(&harness, &user_id, "Alice").await;

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
