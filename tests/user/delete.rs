use axum::http::{StatusCode, header};
use axum_test::TestResponse;

use zekurix_server::user::UserId;

use super::common::UserResponse;
use super::create::create_user;
use super::get::get_user;
use crate::common::{ErrorResponse, TestApplication};

fn assert_problem_not_found(response: TestResponse) {
    response.assert_status_not_found();
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/user/not-found"
    );
    assert_eq!(body.status, StatusCode::NOT_FOUND.as_u16());
}

#[tokio::test]
async fn should_return_not_found_when_deleting_unknown_user() {
    let app = TestApplication::new().await;
    let user_id = UserId::new();

    let response = app
        .server
        .delete(&format!("/api/v1/users/{user_id}"))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;
    assert_problem_not_found(response);
}

#[tokio::test]
async fn should_delete_user_as_admin() {
    let app = TestApplication::new().await;
    let alice = create_user(&app, "Alice").await;

    let response = app
        .server
        .delete(&format!("/api/v1/users/{}", alice.id))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = get_user(&app, alice.id, "admin").await;
    response.assert_status_not_found();
}

#[tokio::test]
async fn should_return_not_found_when_deleting_user_twice() {
    let app = TestApplication::new().await;
    let alice = create_user(&app, "Alice").await;

    let response = app
        .server
        .delete(&format!("/api/v1/users/{}", alice.id))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = app
        .server
        .delete(&format!("/api/v1/users/{}", alice.id))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;
    assert_problem_not_found(response);
}

#[tokio::test]
async fn should_delete_current_user() {
    let app = TestApplication::new().await;
    let alice = create_user(&app, "Alice").await;

    let response = app
        .server
        .delete(&format!("/api/v1/users/{}", alice.id))
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = get_user(&app, alice.id, "admin").await;
    response.assert_status_not_found();
}

#[tokio::test]
async fn should_recreate_user_after_login_following_deletion() {
    let app = TestApplication::new().await;
    let alice = create_user(&app, "Alice").await;

    let response = app
        .server
        .delete(&format!("/api/v1/users/{}", alice.id))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let recreated_alice: UserResponse = response.json();

    assert_ne!(recreated_alice.id, alice.id);

    let response = get_user(&app, recreated_alice.id, "admin").await;
    response.assert_status_ok();
}

#[tokio::test]
async fn should_require_authentication_to_delete_user() {
    let app = TestApplication::new().await;
    let alice = create_user(&app, "Alice").await;

    let response = app
        .server
        .delete(&format!("/api/v1/users/{}", alice.id))
        .await;

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
