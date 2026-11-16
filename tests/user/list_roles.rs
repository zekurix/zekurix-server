use axum::http::{StatusCode, header};
use axum_test::TestResponse;
use serde_json::json;

use zekurix_server::user::UserId;

use crate::{
    authorization::common::create_role,
    common::{TestApplication, assert_unauthorized},
};

use super::{assign_role::assign_role, common::assert_user_not_found, create::create_user};

pub async fn list_roles(app: &TestApplication, user_id: UserId, subject: &str) -> TestResponse {
    app.server
        .get(&format!("/api/v1/users/{user_id}/roles"))
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .await
}

#[tokio::test]
async fn should_list_roles_assigned_to_user() {
    // Create a user with two assigned roles and expect both role summaries.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;
    let reader = create_role(&app, "reader", &[]).await;
    let writer = create_role(&app, "writer", &[]).await;

    assign_role(&app, user.id, reader.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);
    assign_role(&app, user.id, writer.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = list_roles(&app, user.id, "admin").await;

    response.assert_status(StatusCode::OK);
    response.assert_json(&json!({
        "roles": [
            {
                "id": reader.id,
                "name": reader.name,
            },
            {
                "id": writer.id,
                "name": writer.name,
            }
        ]
    }));
}

#[tokio::test]
async fn should_return_empty_list_when_user_has_no_roles() {
    // Create a user without assigned roles and expect an empty role list.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;

    let response = list_roles(&app, user.id, "admin").await;

    response.assert_status(StatusCode::OK);
    response.assert_json(&json!({"roles": []}));
}

#[tokio::test]
async fn should_return_not_found_for_unknown_user() {
    // Request roles for an unknown user and expect a user-not-found response.
    let app = TestApplication::new().await;

    let response = list_roles(&app, UserId::new(), "admin").await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_list_user_roles() {
    // Request user roles without authentication and expect the request to be rejected.
    let app = TestApplication::new().await;
    let user = create_user(&app, "Alice").await;

    let response = app
        .server
        .get(&format!("/api/v1/users/{}/roles", user.id))
        .await;

    assert_unauthorized(&response);
}
