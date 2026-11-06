use axum::http::{StatusCode, header};
use axum_test::TestResponse;

use zekurix_server::user::UserId;

use super::common::UsersResponse;
use super::create::create_users;
use crate::common::{ErrorResponse, TestApplication};

async fn list_users(app: &TestApplication, subject: &str, query: Option<&str>) -> TestResponse {
    let uri = match query {
        Some(query) => format!("/api/v1/users?{query}"),
        None => "/api/v1/users".to_owned(),
    };

    app.server
        .get(&uri)
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .await
}

fn assert_user_ids(response: TestResponse, expected: &[UserId]) {
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

fn assert_bad_request(response: TestResponse) {
    response.assert_status_bad_request();
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(body.status, StatusCode::BAD_REQUEST.as_u16());
    assert!(!body.title.is_empty());
    assert!(!body.detail.is_empty());
}

#[tokio::test]
async fn should_use_default_pagination_when_parameters_are_omitted() {
    let app = TestApplication::new().await;
    let expected = create_users(&app, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&app, "admin", None).await;

    assert_user_ids(response, &expected);
}

#[tokio::test]
async fn should_apply_limit() {
    let app = TestApplication::new().await;
    let users = create_users(&app, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&app, "admin", Some("limit=2")).await;

    assert_user_ids(response, &users[..2]);
}

#[tokio::test]
async fn should_apply_offset() {
    let app = TestApplication::new().await;
    let users = create_users(&app, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&app, "admin", Some("offset=1")).await;

    assert_user_ids(response, &users[1..]);
}

#[tokio::test]
async fn should_apply_limit_and_offset() {
    let app = TestApplication::new().await;
    let users = create_users(&app, &["admin", "Alice", "Bob", "Charlie", "David", "Eve"]).await;

    let response = list_users(&app, "admin", Some("limit=2&offset=1")).await;

    assert_user_ids(response, &users[1..3]);
}

#[tokio::test]
async fn should_return_last_partial_page() {
    let app = TestApplication::new().await;
    let users = create_users(&app, &["admin", "Alice", "Bob", "Charlie", "David", "Eve"]).await;

    let response = list_users(&app, "admin", Some("limit=2&offset=5")).await;

    assert_user_ids(response, &users[5..]);
}

#[tokio::test]
async fn should_return_empty_page_when_offset_equals_user_count() {
    let app = TestApplication::new().await;
    create_users(&app, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&app, "admin", Some("limit=2&offset=4")).await;

    assert_user_ids(response, &[]);
}

#[tokio::test]
async fn should_return_empty_page_when_offset_exceeds_user_count() {
    let app = TestApplication::new().await;
    create_users(&app, &["admin", "Alice", "Bob", "Charlie"]).await;

    let response = list_users(&app, "admin", Some("limit=2&offset=100")).await;

    assert_user_ids(response, &[]);
}

#[tokio::test]
async fn should_accept_minimum_limit() {
    let app = TestApplication::new().await;
    let users = create_users(&app, &["admin", "Alice", "Bob"]).await;

    let response = list_users(&app, "admin", Some("limit=1&offset=0")).await;

    assert_user_ids(response, &users[..1]);
}

#[tokio::test]
async fn should_accept_maximum_limit() {
    let app = TestApplication::new().await;
    let expected = create_users(&app, &["admin", "Alice", "Bob"]).await;

    let response = list_users(&app, "admin", Some("limit=100&offset=0")).await;

    assert_user_ids(response, &expected);
}

#[tokio::test]
async fn should_reject_zero_limit() {
    let app = TestApplication::new().await;

    let response = list_users(&app, "admin", Some("limit=0")).await;

    assert_bad_request(response);
}

#[tokio::test]
async fn should_reject_limit_above_maximum() {
    let app = TestApplication::new().await;

    let response = list_users(&app, "admin", Some("limit=101")).await;

    assert_bad_request(response);
}

#[tokio::test]
async fn should_reject_negative_limit() {
    let app = TestApplication::new().await;

    let response = list_users(&app, "admin", Some("limit=-1")).await;

    assert_bad_request(response);
}

#[tokio::test]
async fn should_reject_non_integer_limit() {
    let app = TestApplication::new().await;

    let response = list_users(&app, "admin", Some("limit=invalid")).await;

    assert_bad_request(response);
}

#[tokio::test]
async fn should_reject_negative_offset() {
    let app = TestApplication::new().await;

    let response = list_users(&app, "admin", Some("offset=-1")).await;

    assert_bad_request(response);
}

#[tokio::test]
async fn should_reject_non_integer_offset() {
    let app = TestApplication::new().await;

    let response = list_users(&app, "admin", Some("offset=invalid")).await;

    assert_bad_request(response);
}

#[tokio::test]
async fn should_ignore_unknown_query_parameters() {
    let app = TestApplication::new().await;
    let expected = create_users(&app, &["admin", "Alice", "Bob"]).await;

    let response = list_users(&app, "admin", Some("limit=50&offset=0&unknown=value")).await;

    assert_user_ids(response, &expected);
}

#[tokio::test]
async fn should_require_authentication_to_list_users() {
    let app = TestApplication::new().await;

    let response = app.server.get("/api/v1/users").await;

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
