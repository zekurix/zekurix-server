use axum::http::StatusCode;
use axum_test::TestResponse;
use serde::Deserialize;

use zekurix_server::authorization::RoleId;

use crate::common::{ErrorResponse, TestApplication};

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

pub fn assert_role_not_found(response: TestResponse) {
    response.assert_status_not_found();
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(
        body.r#type.as_str(),
        "https://api.zekurix.com/problems/role/not-found"
    );
    assert_eq!(body.status, StatusCode::NOT_FOUND.as_u16());
    assert!(!body.title.is_empty());
    assert!(!body.detail.is_empty());
}

pub async fn create_role(app: &TestApplication, name: &str, permissions: &[&str]) -> RoleResponse {
    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(axum::http::header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": name,
            "permissions": permissions
        }))
        .await;

    response.assert_status(axum::http::StatusCode::CREATED);
    response.json()
}
