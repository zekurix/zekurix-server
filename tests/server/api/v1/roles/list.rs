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

    response.assert_status_ok();
    assert_eq!(response.content_type(), "application/json");

    let body: RolesResponse = response.json();

    assert_eq!(
        body.roles
            .into_iter()
            .map(|role| role.id)
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(body.pagination.limit, 50);
    assert_eq!(body.pagination.offset, 0);
    assert_eq!(body.pagination.total, expected.len() as u64);
}

#[tokio::test]
async fn should_filter_roles_by_exact_name() {
    let harness = TestHarness::new().await;

    let reader = create_role(&harness, "reader", &[]).await;
    create_role(&harness, "reader-admin", &[]).await;

    let response = list_roles(&harness, Some("name=reader"), "admin").await;

    response.assert_status_ok();
    let body: RolesResponse = response.json();

    assert_eq!(
        body.roles
            .into_iter()
            .map(|role| role.id)
            .collect::<Vec<_>>(),
        [reader.id]
    );
    assert_eq!(body.pagination.total, 1);
}

#[tokio::test]
async fn should_return_empty_list_when_no_role_matches_name() {
    let harness = TestHarness::new().await;

    create_role(&harness, "reader", &[]).await;

    let response = list_roles(&harness, Some("name=unknown"), "admin").await;

    assert_role_ids(response, &[]);
}

#[tokio::test]
async fn should_apply_limit_and_offset() {
    let harness = TestHarness::new().await;

    let mut roles = existing_role_ids(&harness).await;
    roles.extend(create_roles(&harness, &["one", "two", "three", "four"]).await);
    roles.sort();

    let response = list_roles(&harness, Some("limit=2&offset=1"), "admin").await;

    response.assert_status_ok();
    let body: RolesResponse = response.json();
    assert_eq!(body.pagination.limit, 2);
    assert_eq!(body.pagination.offset, 1);
    assert_eq!(body.pagination.total, roles.len() as u64);

    assert_eq!(
        body.roles
            .into_iter()
            .map(|role| role.id)
            .collect::<Vec<_>>(),
        roles[1..3]
    );
}

#[tokio::test]
async fn should_return_empty_page_when_offset_exceeds_role_count() {
    let harness = TestHarness::new().await;

    let existing_roles = existing_role_ids(&harness).await;
    create_roles(&harness, &["one", "two"]).await;

    let response = list_roles(&harness, Some("limit=2&offset=100"), "admin").await;

    response.assert_status_ok();
    let body: RolesResponse = response.json();

    assert!(body.roles.is_empty());
    assert_eq!(body.pagination.limit, 2);
    assert_eq!(body.pagination.offset, 100);
    assert_eq!(body.pagination.total, (existing_roles.len() + 2) as u64);
}

#[tokio::test]
async fn should_reject_invalid_query() {
    let harness = TestHarness::new().await;

    for query in [
        "limit=0",
        "limit=101",
        "limit=-1",
        "limit=invalid",
        "offset=-1",
        "name=",
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
