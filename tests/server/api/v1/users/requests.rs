use axum::http::header;
use axum_test::TestResponse;
use serde_json::Value;

use crate::server::harness::TestHarness;

pub(super) async fn create_user(harness: &TestHarness, body: Value, actor: &str) -> TestResponse {
    harness
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .json(&body)
        .await
}

pub(super) async fn get_user(harness: &TestHarness, user_id: &str, actor: &str) -> TestResponse {
    harness
        .server
        .get(&format!("/api/v1/users/{user_id}"))
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}

pub(super) async fn get_current_user(harness: &TestHarness, actor: &str) -> TestResponse {
    harness
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}

pub(super) async fn list_users(
    harness: &TestHarness,
    query: Option<&str>,
    actor: &str,
) -> TestResponse {
    let uri = match query {
        Some(query) => format!("/api/v1/users?{query}"),
        None => "/api/v1/users".to_owned(),
    };

    harness
        .server
        .get(&uri)
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}

pub(super) async fn delete_user(harness: &TestHarness, user_id: &str, actor: &str) -> TestResponse {
    harness
        .server
        .delete(&format!("/api/v1/users/{user_id}"))
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}

pub(super) async fn assign_role(
    harness: &TestHarness,
    user_id: &str,
    role_id: &str,
    actor: &str,
) -> TestResponse {
    harness
        .server
        .put(&format!("/api/v1/users/{user_id}/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}

pub(super) async fn unassign_role(
    harness: &TestHarness,
    user_id: &str,
    role_id: &str,
    actor: &str,
) -> TestResponse {
    harness
        .server
        .delete(&format!("/api/v1/users/{user_id}/roles/{role_id}"))
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}

pub(super) async fn list_user_roles(
    harness: &TestHarness,
    user_id: &str,
    actor: &str,
) -> TestResponse {
    harness
        .server
        .get(&format!("/api/v1/users/{user_id}/roles"))
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}

pub(super) async fn list_user_permissions(
    harness: &TestHarness,
    user_id: &str,
    actor: &str,
) -> TestResponse {
    harness
        .server
        .get(&format!("/api/v1/users/{user_id}/permissions"))
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}
