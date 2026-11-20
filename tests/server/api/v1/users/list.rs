use axum_test::TestResponse;

use crate::server::harness::TestHarness;
use crate::server::problem::{assert_http_unauthorized, assert_query_invalid};

use super::fixtures::create_users;
use super::models::UsersResponse;
use super::requests::list_users;

fn assert_user_ids(response: TestResponse, expected: &[String]) {
    response.assert_status_ok();
    assert_eq!(response.content_type(), "application/json");

    let body: UsersResponse = response.json();
    let actual = body
        .users
        .into_iter()
        .map(|user| user.id)
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn should_use_default_pagination_when_parameters_are_omitted() {
    let harness = TestHarness::new().await;
    let expected = create_users(&harness, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&harness, None, "admin").await;

    assert_user_ids(response, &expected);
}

#[tokio::test]
async fn should_apply_limit() {
    let harness = TestHarness::new().await;
    let users = create_users(&harness, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&harness, Some("limit=2"), "admin").await;

    assert_user_ids(response, &users[..2]);
}

#[tokio::test]
async fn should_apply_offset() {
    let harness = TestHarness::new().await;
    let users = create_users(&harness, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&harness, Some("offset=1"), "admin").await;

    assert_user_ids(response, &users[1..]);
}

#[tokio::test]
async fn should_apply_limit_and_offset() {
    let harness = TestHarness::new().await;
    let users = create_users(
        &harness,
        &["admin", "Alice", "Bob", "Charlie", "David", "Eve"],
    )
    .await;

    let response = list_users(&harness, Some("limit=2&offset=1"), "admin").await;

    assert_user_ids(response, &users[1..3]);
}

#[tokio::test]
async fn should_return_last_partial_page() {
    let harness = TestHarness::new().await;
    let users = create_users(
        &harness,
        &["admin", "Alice", "Bob", "Charlie", "David", "Eve"],
    )
    .await;

    let response = list_users(&harness, Some("limit=2&offset=5"), "admin").await;

    assert_user_ids(response, &users[5..]);
}

#[tokio::test]
async fn should_return_empty_page_when_offset_equals_user_count() {
    let harness = TestHarness::new().await;

    create_users(&harness, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&harness, Some("limit=2&offset=4"), "admin").await;

    assert_user_ids(response, &[]);
}

#[tokio::test]
async fn should_return_empty_page_when_offset_exceeds_user_count() {
    let harness = TestHarness::new().await;

    create_users(&harness, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&harness, Some("limit=2&offset=100"), "admin").await;

    assert_user_ids(response, &[]);
}

#[tokio::test]
async fn should_accept_minimum_limit() {
    let harness = TestHarness::new().await;
    let users = create_users(&harness, &["admin", "Alice", "Bob"]).await;

    let response = list_users(&harness, Some("limit=1&offset=0"), "admin").await;

    assert_user_ids(response, &users[..1]);
}

#[tokio::test]
async fn should_accept_maximum_limit() {
    let harness = TestHarness::new().await;
    let expected = create_users(&harness, &["admin", "Alice", "Bob"]).await;

    let response = list_users(&harness, Some("limit=100&offset=0"), "admin").await;

    assert_user_ids(response, &expected);
}

#[tokio::test]
async fn should_ignore_unknown_query_parameters() {
    let harness = TestHarness::new().await;
    let expected = create_users(&harness, &["admin", "Alice", "Bob"]).await;

    let response = list_users(&harness, Some("limit=50&offset=0&unknown=value"), "admin").await;

    assert_user_ids(response, &expected);
}

#[tokio::test]
async fn should_reject_zero_limit() {
    let harness = TestHarness::new().await;

    let response = list_users(&harness, Some("limit=0"), "admin").await;

    assert_query_invalid(&response);
}

#[tokio::test]
async fn should_reject_limit_above_maximum() {
    let harness = TestHarness::new().await;

    let response = list_users(&harness, Some("limit=101"), "admin").await;

    assert_query_invalid(&response);
}

#[tokio::test]
async fn should_reject_negative_limit() {
    let harness = TestHarness::new().await;

    let response = list_users(&harness, Some("limit=-1"), "admin").await;

    assert_query_invalid(&response);
}

#[tokio::test]
async fn should_reject_non_integer_limit() {
    let harness = TestHarness::new().await;

    let response = list_users(&harness, Some("limit=invalid"), "admin").await;

    assert_query_invalid(&response);
}

#[tokio::test]
async fn should_reject_negative_offset() {
    let harness = TestHarness::new().await;

    let response = list_users(&harness, Some("offset=-1"), "admin").await;

    assert_query_invalid(&response);
}

#[tokio::test]
async fn should_reject_non_integer_offset() {
    let harness = TestHarness::new().await;

    let response = list_users(&harness, Some("offset=invalid"), "admin").await;

    assert_query_invalid(&response);
}

#[tokio::test]
async fn should_require_authentication_to_list_users() {
    let harness = TestHarness::new().await;

    let response = harness.server.get("/api/v1/users").await;

    assert_http_unauthorized(&response);
}
