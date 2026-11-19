use axum::http::{StatusCode, header};
use axum_test::TestResponse;
use serde::Deserialize;

use zekurix_server::authorization::RoleId;

use crate::common::{TestApplication, assert_problem};
use crate::user::assign_role::assign_role;
use crate::user::create::create_user;

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoleResponse {
    pub id: RoleId,
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolesResponse {
    pub roles: Vec<RoleResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PermissionsResponse {
    permissions: Vec<String>,
}

pub fn assert_role_not_found(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::NOT_FOUND,
        "https://api.zekurix.com/problems/role/not-found",
    );
}

pub fn assert_role_already_exists(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::CONFLICT,
        "https://api.zekurix.com/problems/role/already-exists",
    );
}

pub fn assert_duplicate_permission(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "https://api.zekurix.com/problems/permission/duplicate",
    );
}

pub async fn list_permissions(app: &TestApplication) -> TestResponse {
    app.server
        .get("/api/v1/permissions")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await
}

pub async fn create_role(app: &TestApplication, name: &str, permissions: &[&str]) -> RoleResponse {
    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": name,
            "permissions": permissions
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    response.json()
}

pub async fn get_role(app: &TestApplication, role_id: RoleId, subject: &str) -> TestResponse {
    app.server
        .get(&format!("/api/v1/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .await
}

pub async fn create_user_without_permissions(app: &TestApplication, subject: &str) {
    create_user(app, subject).await;
}

pub async fn create_user_with_only_permission(
    app: &TestApplication,
    subject: &str,
    permission: &str,
) {
    let user = create_user(app, subject).await;
    let role_name = format!("role-with-only-{permission}");
    let role = create_role(app, &role_name, &[permission]).await;

    assign_role(app, user.id, role.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);
}

pub async fn create_user_with_all_permissions_except(
    app: &TestApplication,
    subject: &str,
    excluded_permission: &str,
) {
    let user = create_user(app, subject).await;

    let response = list_permissions(app).await;

    let all_permissions = response.json::<PermissionsResponse>().permissions;
    let permissions = all_permissions
        .iter()
        .map(String::as_str)
        .filter(|permission| *permission != excluded_permission)
        .collect::<Vec<_>>();
    assert!(
        permissions.len() < all_permissions.len(),
        "permission {excluded_permission} must exist"
    );

    let role_name = format!("role-with-all-permissions-except-{excluded_permission}");
    let role = create_role(app, &role_name, &permissions).await;
    assign_role(app, user.id, role.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);
}
