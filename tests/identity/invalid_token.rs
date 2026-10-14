use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use test_case::test_case;

use crate::common::TestApplication;

use super::common::assert_invalid_auth_header;
use super::common::build_valid_claims;
use super::common::build_valid_header;

#[test_case(""; "empty token")]
#[test_case("invalid"; "invalid token")]
#[test_case("header.payload"; "missing token segment")]
#[test_case("header.payload.signature.extra"; "too many token segments")]
#[test_case(".."; "empty segments")]
#[test_case("header..signature"; "empty payload")]
#[test_case("header.payload."; "empty signature")]
#[tokio::test]
async fn should_reject_invalid_token(token: &str) {
    let app = TestApplication::new().await;

    let header = build_valid_header(token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_invalid_signature() {
    let app = TestApplication::new().await;

    let claims = build_valid_claims(app.auth.clone(), "Alice");
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret("invalid key".as_bytes()),
    )
    .expect("failed to generate test JWT");
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_unexpected_algorithm() {
    let app = TestApplication::new().await;

    let claims = build_valid_claims(app.auth.clone(), "Alice");
    let token = encode(
        &Header::new(Algorithm::HS384),
        &claims,
        &EncodingKey::from_secret(app.auth.secret.as_bytes()),
    )
    .expect("failed to generate test JWT");
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}
