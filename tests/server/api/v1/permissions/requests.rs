use axum::http::header;
use axum_test::TestResponse;

use crate::server::harness::TestHarness;

pub(crate) async fn list_permissions(harness: &TestHarness, actor: &str) -> TestResponse {
    harness
        .server
        .get("/api/v1/permissions")
        .add_header(header::AUTHORIZATION, harness.authentication.header(actor))
        .await
}
