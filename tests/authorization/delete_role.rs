use axum::http::{StatusCode, header};
use axum_test::TestResponse;

use zekurix_server::authorization::RoleId;

use crate::common::{TestApplication, assert_unauthorized};

use super::common::{assert_role_not_found, create_role, get_role};

async fn delete_role(app: &TestApplication, role_id: RoleId) -> TestResponse {
    app.server
        .delete(&format!("/api/v1/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await
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
