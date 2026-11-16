use axum::http::{StatusCode, header};
use axum_test::TestResponse;
use serde_json::json;

use zekurix_server::authorization::RoleId;
use zekurix_server::user::UserId;

use crate::authorization::common::{assert_role_not_found, create_role};
use crate::common::{TestApplication, assert_unauthorized};

use super::{
    assign_role::assign_role, common::assert_user_not_found, create::create_user,
    list_roles::list_roles,
};

pub async fn unassign_role(
    app: &TestApplication,
    user_id: UserId,
    role_id: RoleId,
    subject: &str,
) -> TestResponse {
    app.server
        .delete(&format!("/api/v1/users/{user_id}/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .await
}

#[tokio::test]
async fn should_unassign_role_from_user() {
    // Assign a role to a user, unassign it, and expect the user to have no roles.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;
    let role = create_role(&app, "reader", &[]).await;

    assign_role(&app, user.id, role.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = unassign_role(&app, user.id, role.id, "admin").await;

    response.assert_status(StatusCode::NO_CONTENT);

    let response = list_roles(&app, user.id, "admin").await;

    response.assert_status(StatusCode::OK);
    response.assert_json(&json!({"roles": []}));
}

#[tokio::test]
async fn should_be_idempotent_when_role_is_not_assigned() {
    // Unassign an existing but unassigned role twice and expect both requests to succeed.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;
    let role_id = create_role(&app, "reader", &[]).await.id;

    unassign_role(&app, user.id, role_id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    unassign_role(&app, user.id, role_id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    // Unassign a role from an unknown user and expect a user-not-found response.
    let app = TestApplication::new().await;
    let role_id = create_role(&app, "reader", &[]).await.id;

    let response = unassign_role(&app, UserId::new(), role_id, "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_return_not_found_for_unknown_role() {
    // Unassign an unknown role from an existing user and expect a role-not-found response.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;

    let response = unassign_role(&app, user.id, RoleId::new(), "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_unassign_role() {
    // Unassign a role without authentication and expect the request to be rejected.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;
    let role_id = create_role(&app, "reader", &[]).await.id;

    let response = app
        .server
        .delete(&format!("/api/v1/users/{}/roles/{role_id}", user.id))
        .await;

    assert_unauthorized(&response);
}
