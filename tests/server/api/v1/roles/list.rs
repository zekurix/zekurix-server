use axum_test::TestResponse;

use crate::server::api::v1::users::fixtures::{
    create_user_with_all_permissions_except, create_user_with_only_permission,
    create_user_without_permissions,
};
use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_http_forbidden, assert_http_unauthorized, assert_query_invalid,
};

use super::fixtures::create_role;
use super::models::RolesResponse;
use super::requests::list_roles;

fn role_ids(response: TestResponse) -> Vec<String> {
    response.assert_status_ok();
    assert_eq!(response.content_type(), "application/json");

    let body: RolesResponse = response.json();

    body.roles.into_iter().map(|role| role.id).collect()
}

fn assert_role_ids(response: TestResponse, expected: &[String]) {
    let actual = role_ids(response);

    assert_eq!(actual, expected);
}

async fn existing_role_ids(harness: &TestHarness) -> Vec<String> {
    role_ids(list_roles(harness, None, "admin").await)
}

async fn create_roles(harness: &TestHarness, names: &[&str]) -> Vec<String> {
    let mut ids = Vec::with_capacity(names.len());

    for name in names {
        ids.push(create_role(harness, name, &[]).await.id);
    }

    // Match the API's ordering by role ID.
    ids.sort();
    ids
}

#[tokio::test]
async fn should_list_roles_with_default_pagination() {
    let harness = TestHarness::new().await;

    let mut expected = existing_role_ids(&harness).await;
    expected.extend(create_roles(&harness, &["reader", "editor", "manager"]).await);
    expected.sort();

    let response = list_roles(&harness, None, "admin").await;

    assert_role_ids(response, &expected);
}

#[tokio::test]
async fn should_apply_limit_and_offset() {
    let harness = TestHarness::new().await;

    let mut roles = existing_role_ids(&harness).await;
    roles.extend(create_roles(&harness, &["one", "two", "three", "four"]).await);
    roles.sort();

    let response = list_roles(&harness, Some("limit=2&offset=1"), "admin").await;

    assert_role_ids(response, &roles[1..3]);
}

#[tokio::test]
async fn should_return_an_empty_page_when_offset_exceeds_role_count() {
    let harness = TestHarness::new().await;

    create_roles(&harness, &["one", "two"]).await;

    let response = list_roles(&harness, Some("limit=2&offset=100"), "admin").await;

    assert_role_ids(response, &[]);
}

#[tokio::test]
async fn should_reject_invalid_pagination() {
    let harness = TestHarness::new().await;

    for query in [
        "limit=0",
        "limit=101",
        "limit=-1",
        "limit=invalid",
        "offset=-1",
    ] {
        let response = list_roles(&harness, Some(query), "admin").await;

        assert_query_invalid(&response);
    }
}

#[tokio::test]
async fn should_require_authentication_to_list_roles() {
    let harness = TestHarness::new().await;

    let response = harness.server.get("/api/v1/roles").await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_list_roles_for_user_with_only_roles_read_permission() {
    let harness = TestHarness::new().await;
    let actor = create_user_with_only_permission(&harness, "roles:read").await;

    let mut expected = existing_role_ids(&harness).await;
    expected.extend(create_roles(&harness, &["reader", "editor"]).await);
    expected.sort();

    let response = list_roles(&harness, None, &actor).await;

    assert_role_ids(response, &expected);
}

#[tokio::test]
async fn should_reject_role_listing_for_user_without_permissions() {
    let harness = TestHarness::new().await;
    let actor = create_user_without_permissions(&harness).await;

    let response = list_roles(&harness, None, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_role_listing_for_user_without_roles_read_permission() {
    let harness = TestHarness::new().await;
    let actor = create_user_with_all_permissions_except(&harness, "roles:read").await;

    let response = list_roles(&harness, None, &actor).await;

    assert_http_forbidden(&response);
}
