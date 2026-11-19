use axum::http::{StatusCode, header};
use axum_test::TestResponse;

use zekurix_server::authorization::RoleId;

use crate::common::{TestApplication, assert_forbidden, assert_unauthorized};

use super::common::{
    assert_role_not_found, create_role, create_user_with_all_permissions_except,
    create_user_with_only_permission, create_user_without_permissions, get_role,
};

async fn delete_role_as(app: &TestApplication, role_id: RoleId, subject: &str) -> TestResponse {
    app.server
        .delete(&format!("/api/v1/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .await
}

async fn delete_role(app: &TestApplication, role_id: RoleId) -> TestResponse {
    delete_role_as(app, role_id, "admin").await
}

#[tokio::test]
async fn should_delete_role() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &["roles:read"]).await;

    let response = delete_role(&app, role.id).await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = get_role(&app, role.id, "admin").await;
    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_return_not_found_when_deleting_role_twice() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &["roles:read"]).await;

    let response = delete_role(&app, role.id).await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = delete_role(&app, role.id).await;
    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_return_not_found_when_deleting_unknown_role() {
    let app = TestApplication::new().await;

    let response = delete_role(&app, RoleId::new()).await;
    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_delete_role() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;

    let response = app
        .server
        .delete(&format!("/api/v1/roles/{}", role.id))
        .await;
    assert_unauthorized(&response);
}

#[tokio::test]
async fn should_delete_role_for_user_with_only_roles_delete_permission() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;
    let subject = "user-with-only-roles-delete";
    create_user_with_only_permission(&app, subject, "roles:delete").await;

    let response = delete_role_as(&app, role.id, subject).await;
    response.assert_status(StatusCode::NO_CONTENT);

    let response = get_role(&app, role.id, "admin").await;
    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_reject_role_deletion_for_user_without_permissions() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;
    let subject = "user-without-permissions";
    create_user_without_permissions(&app, subject).await;

    let response = delete_role_as(&app, role.id, subject).await;

    assert_forbidden(&response);
}

#[tokio::test]
async fn should_reject_role_deletion_for_user_without_roles_delete_permission() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;
    let subject = "user-without-roles-delete";
    create_user_with_all_permissions_except(&app, subject, "roles:delete").await;

    let response = delete_role_as(&app, role.id, subject).await;

    assert_forbidden(&response);
}
