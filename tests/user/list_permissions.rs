use axum::http::{StatusCode, header};
use axum_test::TestResponse;
use serde_json::json;

use zekurix_server::user::UserId;

use crate::{
    authorization::common::create_role,
    common::{TestApplication, assert_unauthorized},
};

use super::{assign_role::assign_role, common::assert_user_not_found, create::create_user};

pub async fn list_user_permissions(
    app: &TestApplication,
    user_id: UserId,
    subject: &str,
) -> TestResponse {
    app.server
        .get(&format!("/api/v1/users/{user_id}/permissions"))
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .await
}

#[tokio::test]
async fn should_list_permissions_granted_to_user() {
    // Assign roles with overlapping permissions and expect the effective permission set.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;

    let reader = create_role(&app, "reader", &["roles:read"]).await;
    let editor = create_role(&app, "editor", &["roles:update"]).await;

    assign_role(&app, user.id, reader.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);
    assign_role(&app, user.id, editor.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = list_user_permissions(&app, user.id, "admin").await;

    response.assert_status(StatusCode::OK);
    response.assert_json(&json!({
        "permissions": [
            "roles:read",
            "roles:update"
        ]
    }));
}

#[tokio::test]
async fn should_list_distinct_permissions_from_multiple_roles() {
    // Assign two roles with overlapping permissions and expect their union without duplicates.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;

    let reader = create_role(&app, "reader", &["roles:read", "roles:update"]).await;

    let admin = create_role(&app, "admin-role", &["roles:update", "roles:delete"]).await;

    assign_role(&app, user.id, reader.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    assign_role(&app, user.id, admin.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = list_user_permissions(&app, user.id, "admin").await;

    response.assert_status(StatusCode::OK);
    response.assert_json(&json!({
        "permissions": [
            "roles:delete",
            "roles:read",
            "roles:update"
        ]
    }));
}

#[tokio::test]
async fn should_return_empty_list_when_user_has_no_permissions() {
    // Create a user without assigned roles and expect an empty permission list.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;

    let response = list_user_permissions(&app, user.id, "admin").await;

    response.assert_status(StatusCode::OK);
    response.assert_json(&json!({
        "permissions": []
    }));
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    // Request permissions for an unknown user and expect a user-not-found response.
    let app = TestApplication::new().await;

    let response = list_user_permissions(&app, UserId::new(), "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_list_user_permissions() {
    // Request user permissions without authentication and expect the request to be rejected.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;

    let response = app
        .server
        .get(&format!("/api/v1/users/{}/permissions", user.id))
        .await;

    assert_unauthorized(&response);
}
