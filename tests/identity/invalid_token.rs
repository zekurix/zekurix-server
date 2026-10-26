use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use test_case::test_case;

use crate::common::TestApplication;

use super::common::assert_invalid_auth_header;
use super::common::build_valid_claims;
use super::common::build_valid_header;
use super::common::build_valid_token;

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

    let token = build_valid_token(app.auth.clone(), claims);
    let mut parts: Vec<String> = token.split('.').map(str::to_owned).collect();
    parts[2].push('A'); // Corrupt the signature
    let token = parts.join(".");

    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_unexpected_algorithm() {
    let app = TestApplication::new().await;

    let claims = build_valid_claims(app.auth.clone(), "Alice");

    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some(app.auth.kid.clone());
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret("invalid key".as_bytes()),
    )
    .expect("failed to generate test JWT");
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_unknown_kid() {
    let app = TestApplication::new().await;

    let claims = build_valid_claims(app.auth.clone(), "Alice");

    let mut header = Header::new(app.auth.algorithm);
    header.kid = Some("unknown".to_string());
    let token =
        encode(&header, &claims, &app.auth.encoding_key).expect("failed to generate test JWT");

    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}
