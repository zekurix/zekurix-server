use axum::http::{StatusCode, header};
use axum_test::TestResponse;

use zekurix_server::authorization::RoleId;
use zekurix_server::user::UserId;

use crate::authorization::common::{assert_role_not_found, create_role};
use crate::common::{TestApplication, assert_unauthorized};

use super::common::assert_user_not_found;
use super::create::create_user;

pub async fn assign_role(
    app: &TestApplication,
    user_id: UserId,
    role_id: RoleId,
    subject: &str,
) -> TestResponse {
    app.server
        .put(&format!("/api/v1/users/{user_id}/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .await
}

#[tokio::test]
async fn should_assign_role_to_user() {
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;
    let role_id = create_role(&app, "reader", &[]).await.id;

    let response = assign_role(&app, user.id, role_id, "admin").await;

    response.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn should_be_idempotent_when_role_is_already_assigned() {
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;
    let role_id = create_role(&app, "reader", &[]).await.id;

    let response = assign_role(&app, user.id, role_id, "admin").await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = assign_role(&app, user.id, role_id, "admin").await;
    response.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    let app = TestApplication::new().await;
    let role_id = create_role(&app, "reader", &[]).await.id;

    let response = assign_role(&app, UserId::new(), role_id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_return_not_found_for_unknown_role() {
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;

    let response = assign_role(&app, user.id, RoleId::new(), "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_assign_role() {
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;
    let role_id = create_role(&app, "reader", &[]).await.id;

    let response = app
        .server
        .put(&format!("/api/v1/users/{}/roles/{role_id}", user.id))
        .await;

    assert_unauthorized(&response);
}
