use axum::http::StatusCode;
use serde_json::json;

use crate::server::harness::TestHarness;

use super::models::RoleResponse;
use super::requests::create_role as request_create_role;

pub(crate) async fn create_role(
    harness: &TestHarness,
    name: &str,
    permissions: &[&str],
) -> RoleResponse {
    let body = json!({
        "name": name,
        "permissions": permissions,
    });

    let response = request_create_role(harness, body, "admin").await;

    response.assert_status(StatusCode::CREATED);
    response.json()
}
