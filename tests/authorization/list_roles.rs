use axum::http::{StatusCode, header};
use axum_test::TestResponse;

use zekurix_server::authorization::RoleId;

use super::common::{RolesResponse, create_role};
use crate::common::{ErrorResponse, TestApplication};

async fn list_roles(app: &TestApplication, query: Option<&str>) -> TestResponse {
    let uri = match query {
        Some(query) => format!("/api/v1/roles?{query}"),
        None => "/api/v1/roles".to_owned(),
    };

    app.server
        .get(&uri)
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await
}

fn assert_role_ids(response: TestResponse, expected: &[RoleId]) {
    response.assert_status_ok();
    assert_eq!(response.content_type(), "application/json");

    let body: RolesResponse = response.json();
    let actual = body
        .roles
        .into_iter()
        .map(|role| role.id)
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

fn assert_bad_request(response: TestResponse) {
    response.assert_status_bad_request();
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(body.status, StatusCode::BAD_REQUEST.as_u16());
    assert!(!body.title.is_empty());
    assert!(!body.detail.is_empty());
}

async fn create_roles(app: &TestApplication, names: &[&str]) -> Vec<RoleId> {
    let mut ids = Vec::with_capacity(names.len());
    for name in names {
        ids.push(create_role(app, name, &[]).await.id);
    }

    // The repository returns roles ordered by their ID.
    ids.sort();
    ids
}

#[tokio::test]
async fn should_list_roles_with_default_pagination() {
    let app = TestApplication::new().await;
    let expected = create_roles(&app, &["reader", "editor", "manager"]).await;

    let response = list_roles(&app, None).await;

    assert_role_ids(response, &expected);
}

#[tokio::test]
async fn should_apply_limit_and_offset() {
    let app = TestApplication::new().await;
    let roles = create_roles(&app, &["one", "two", "three", "four"]).await;

    let response = list_roles(&app, Some("limit=2&offset=1")).await;

    assert_role_ids(response, &roles[1..3]);
}

#[tokio::test]
async fn should_return_an_empty_page_when_offset_exceeds_role_count() {
    let app = TestApplication::new().await;
    create_roles(&app, &["one", "two"]).await;

    let response = list_roles(&app, Some("limit=2&offset=100")).await;

    assert_role_ids(response, &[]);
}

#[tokio::test]
async fn should_reject_invalid_pagination() {
    let app = TestApplication::new().await;

    for query in [
        "limit=0",
        "limit=101",
        "limit=-1",
        "limit=invalid",
        "offset=-1",
    ] {
        let response = list_roles(&app, Some(query)).await;
        assert_bad_request(response);
    }
}

#[tokio::test]
async fn should_require_authentication_to_list_roles() {
    let app = TestApplication::new().await;

    let response = app.server.get("/api/v1/roles").await;

    response.assert_status_unauthorized();
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/http/unauthorized"
    );
    assert!(!body.title.is_empty());
    assert_eq!(body.status, StatusCode::UNAUTHORIZED.as_u16());
    assert!(!body.detail.is_empty());
}
