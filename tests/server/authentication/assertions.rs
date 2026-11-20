use axum::http::header;

use crate::server::harness::TestHarness;
use crate::server::problem::assert_http_unauthorized;

pub(super) async fn assert_valid_authentication_header(harness: &TestHarness, value: &str) {
    let response = harness
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, value)
        .await;

    response.assert_status_ok();
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());
}

pub(super) async fn assert_invalid_authentication_header(harness: &TestHarness, value: &str) {
    let response = harness
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, value)
        .await;

    assert_http_unauthorized(&response);
}
