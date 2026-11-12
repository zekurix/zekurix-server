use axum::http::{StatusCode, header};
use axum_test::TestResponse;
use serde::Deserialize;

use zekurix_server::authorization::RoleId;

use crate::common::{TestApplication, assert_problem};

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
