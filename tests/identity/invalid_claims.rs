use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::common::TestApplication;

use super::common::TestClaims;
use super::common::assert_invalid_auth_header;
use super::common::build_valid_claims;
use super::common::build_valid_header;
use super::common::build_valid_token;

#[tokio::test]
async fn should_reject_token_with_empty_claims() {
    let app = TestApplication::new().await;

    let claims = TestClaims {
        iss: None,
        sub: None,
        aud: None,
        exp: None,
        nbf: None,
        other: None,
    };
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_without_issuer() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.iss = None;
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_invalid_issuer() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.iss = Some("invalid issuer".to_string());
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_empty_issuer() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.iss = Some("".to_string());
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_too_long_issuer() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.iss = Some("a".repeat(1025));
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_without_subject() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.sub = None;
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_empty_subject() {
    let app = TestApplication::new().await;

    let claims = build_valid_claims(app.auth.clone(), "");
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_too_long_subject() {
    let app = TestApplication::new().await;

    let claims = build_valid_claims(app.auth.clone(), &"a".repeat(1025));
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_without_audience() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.aud = None;
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_invalid_audience() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.aud = Some("invalid audience".to_string());
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_without_expiration() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.exp = None;
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_expired_token() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.exp = Some(
        SystemTime::now()
            .checked_sub(app.auth.clock_skew + Duration::from_secs(1))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_reject_token_before_not_before() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.nbf = Some(
        SystemTime::now()
            .checked_add(app.auth.clock_skew + Duration::from_secs(1))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_invalid_auth_header(app, &header).await;
}
