use zekurix_server::authorization::RoleId;

use crate::common::{TestApplication, assert_unauthorized};

use super::common::{RoleResponse, assert_role_not_found, create_role, get_role};

#[tokio::test]
async fn should_get_role() {
    let app = TestApplication::new().await;
    let created = create_role(&app, "reader", &["roles:read"]).await;

    let response = get_role(&app, created.id, "admin").await;

    response.assert_status_ok();
    assert_eq!(response.content_type(), "application/json");
    let role: RoleResponse = response.json();
    assert_eq!(role, created);
}

#[tokio::test]
async fn should_get_multiple_roles() {
    let app = TestApplication::new().await;

    let reader = create_role(&app, "reader", &["roles:read"]).await;
    let writer = create_role(&app, "writer", &["roles:update"]).await;
    let admin = create_role(&app, "admin", &["roles:read", "roles:update"]).await;

    for expected in [reader, writer, admin] {
        let response = get_role(&app, expected.id, "admin").await;

        response.assert_status_ok();
        assert_eq!(response.content_type(), "application/json");

        let role: RoleResponse = response.json();
        assert_eq!(role, expected);
    }
}

#[tokio::test]
async fn should_return_not_found_for_unknown_role() {
    let app = TestApplication::new().await;

    let response = get_role(&app, RoleId::new(), "admin").await;

    assert_role_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_get_role() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;

    let response = app.server.get(&format!("/api/v1/roles/{}", role.id)).await;

    assert_unauthorized(&response);
}
