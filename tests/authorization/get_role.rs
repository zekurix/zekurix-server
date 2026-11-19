use zekurix_server::authorization::RoleId;

use crate::common::{TestApplication, assert_forbidden, assert_unauthorized};

use super::common::{
    RoleResponse, assert_role_not_found, create_role, create_user_with_all_permissions_except,
    create_user_with_only_permission, create_user_without_permissions, get_role,
};

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
async fn should_get_role_for_user_with_only_roles_read_permission() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;
    let subject = "user-with-only-roles-read";
    create_user_with_only_permission(&app, subject, "roles:read").await;

    let response = get_role(&app, role.id, subject).await;

    response.assert_status_ok();
    assert_eq!(response.json::<RoleResponse>(), role);
}

#[tokio::test]
async fn should_reject_get_role_for_user_without_permissions() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;
    let subject = "user-without-permissions";
    create_user_without_permissions(&app, subject).await;

    let response = get_role(&app, role.id, subject).await;

    assert_forbidden(&response);
}

#[tokio::test]
async fn should_reject_get_role_for_user_without_roles_read_permission() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;
    let subject = "user-without-roles-read";
    create_user_with_all_permissions_except(&app, subject, "roles:read").await;

    let response = get_role(&app, role.id, subject).await;

    assert_forbidden(&response);
}

#[tokio::test]
async fn should_require_authentication_to_get_role() {
    let app = TestApplication::new().await;
    let role = create_role(&app, "reader", &[]).await;

    let response = app.server.get(&format!("/api/v1/roles/{}", role.id)).await;

    assert_unauthorized(&response);
}
