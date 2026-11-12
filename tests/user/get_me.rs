use axum::http::header;

use crate::common::{TestApplication, assert_unauthorized};

use super::common::UserResponse;
use super::create::create_user;

#[tokio::test]
async fn should_get_current_user_after_admin_creation() {
    let app = TestApplication::new().await;
    let created = create_user(&app, "Alice").await;

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let current: UserResponse = response.json();
    assert_eq!(current, created);
}

#[tokio::test]
async fn should_create_current_user_just_in_time() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let current_user: UserResponse = response.json();

    let response = app
        .server
        .get(&format!("/api/v1/users/{}", current_user.id))
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let persisted_user: UserResponse = response.json();
    assert_eq!(persisted_user.id, current_user.id);
}

#[tokio::test]
async fn should_return_same_user_for_multiple_get_me_requests() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let first_user: UserResponse = response.json();

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let second_user: UserResponse = response.json();
    assert_eq!(second_user.id, first_user.id);
}

#[tokio::test]
async fn should_return_different_users_for_different_subjects() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
    let alice: UserResponse = response.json();

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Bob"))
        .await;
    response.assert_status_ok();
    let bob: UserResponse = response.json();
    assert_ne!(alice.id, bob.id);
}

#[tokio::test]
async fn should_require_authentication_for_get_current_user() {
    let app = TestApplication::new().await;

    let response = app.server.get("/api/v1/users/me").await;

    assert_unauthorized(&response);
}
