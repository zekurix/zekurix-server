use axum::http::{HeaderValue, header};
use test_case::test_case;

use crate::common::TestApplication;

use super::common::assert_invalid_auth_header;
use super::common::assert_unauthorized;
use super::common::build_valid_claims;
use super::common::build_valid_token;

#[tokio::test]
async fn should_reject_missing_auth_header() {
    let app = TestApplication::new().await;

    let response = app.server.get("/api/v1/users/me").await;
    assert_unauthorized(response);
}

#[tokio::test]
async fn should_reject_auth_header_with_invalid_utf8() {
    let app = TestApplication::new().await;

    let header = HeaderValue::from_bytes(b"Bearer \xFF").unwrap();

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, header)
        .await;
    assert_unauthorized(response);
}

#[tokio::test]
async fn should_reject_empty_auth_header() {
    let app = TestApplication::new().await;

    assert_invalid_auth_header(app, "").await;
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
async fn should_reject_invalid_auth_header(prefix: &str) {
    let app = TestApplication::new().await;

    let claims = build_valid_claims(app.auth.clone(), "Alice");
    let token = build_valid_token(app.auth.clone(), claims);
    let header = format!("{prefix}{token}");

    assert_invalid_auth_header(app, &header).await;
}
