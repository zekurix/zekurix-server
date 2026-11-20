use axum::http::{HeaderValue, header};
use test_case::test_case;

use crate::server::harness::TestHarness;
use crate::server::problem::assert_http_unauthorized;

use super::assertions::assert_invalid_authentication_header;

#[tokio::test]
async fn should_reject_missing_authentication_header() {
    let harness = TestHarness::new().await;

    let response = harness.server.get("/api/v1/users/me").await;
    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_reject_authentication_header_with_invalid_utf8() {
    let harness = TestHarness::new().await;

    let header = HeaderValue::from_bytes(b"Bearer \xFF").unwrap();

    let response = harness
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, header)
        .await;
    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_reject_empty_authentication_header() {
    let harness = TestHarness::new().await;

    assert_invalid_authentication_header(&harness, "").await;
}

#[test_case(""; "missing bearer prefix")]
#[test_case(" "; "space bearer prefix")]
#[test_case("Bearer"; "missing space")]
#[test_case("Bearer: "; "invalid prefix format")]
#[test_case("Foo "; "invalid prefix")]
#[test_case("bearer "; "lowercase bearer")]
#[test_case("BEARER "; "uppercase bearer")]
#[test_case("Bearer Foo "; "multi-words prefix")]
#[tokio::test]
async fn should_reject_invalid_authentication_header(prefix: &str) {
    let harness = TestHarness::new().await;

    let token = harness.authentication.token("Alice");
    let header = format!("{prefix}{token}");

    assert_invalid_authentication_header(&harness, &header).await;
}
