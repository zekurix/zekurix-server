use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use test_case::test_case;

use crate::server::harness::TestHarness;

use super::assertions::assert_invalid_authentication_header;

#[test_case(""; "empty token")]
#[test_case("invalid"; "invalid token")]
#[test_case("header.payload"; "missing token segment")]
#[test_case("header.payload.signature.extra"; "too many token segments")]
#[test_case(".."; "empty segments")]
#[test_case("header..signature"; "empty payload")]
#[test_case("header.payload."; "empty signature")]
#[tokio::test]
async fn should_reject_invalid_token(token: &str) {
    let harness = TestHarness::new().await;

    let header = harness.authentication.header_from_token(token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_invalid_signature() {
    let harness = TestHarness::new().await;

    let claims = harness.authentication.valid_claims("Alice");
    let token = harness.authentication.token_with_claims(&claims);

    let mut parts: Vec<String> = token.split('.').map(str::to_owned).collect();
    parts[2].push('A'); // Corrupt the signature
    let token = parts.join(".");

    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_unexpected_algorithm() {
    let harness = TestHarness::new().await;

    let claims = harness.authentication.valid_claims("Alice");

    let mut jwt_header = Header::new(Algorithm::HS256);
    jwt_header.kid = Some(harness.authentication.kid().to_string());

    let token = encode(
        &jwt_header,
        &claims,
        &EncodingKey::from_secret(b"invalid key").expect("failed to create encoding key"),
    )
    .expect("failed to generate test JWT");

    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_unknown_kid() {
    let harness = TestHarness::new().await;

    let claims = harness.authentication.valid_claims("Alice");

    let mut jwt_header = Header::new(harness.authentication.algorithm());
    jwt_header.kid = Some("unknown".to_string());

    let token = harness
        .authentication
        .token_with_header(jwt_header, &claims);

    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}
