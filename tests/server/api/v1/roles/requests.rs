use axum::http::header;
use axum_test::TestResponse;
use serde_json::Value;

use crate::server::harness::TestHarness;

pub(super) async fn create_role(harness: &TestHarness, body: Value, actor: &str) -> TestResponse {
    harness
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .json(&body)
        .await
}

pub(super) async fn get_role(harness: &TestHarness, role_id: &str, actor: &str) -> TestResponse {
    harness
        .server
        .get(&format!("/api/v1/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}

pub(super) async fn list_roles(
    harness: &TestHarness,
    query: Option<&str>,
    actor: &str,
) -> TestResponse {
    let uri = match query {
        Some(query) => format!("/api/v1/roles?{query}"),
        None => "/api/v1/roles".to_owned(),
    };

    harness
        .server
        .get(&uri)
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}

pub(super) async fn update_role(
    harness: &TestHarness,
    role_id: &str,
    body: Value,
    actor: &str,
) -> TestResponse {
    harness
        .server
        .put(&format!("/api/v1/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .json(&body)
        .await
}

pub(super) async fn delete_role(harness: &TestHarness, role_id: &str, actor: &str) -> TestResponse {
    harness
        .server
        .delete(&format!("/api/v1/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}
