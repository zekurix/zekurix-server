use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::server::harness::TestHarness;

use super::assertions::assert_invalid_authentication_header;
use super::test_authentication::TestClaims;

#[tokio::test]
async fn should_reject_token_with_empty_claims() {
    let harness = TestHarness::new().await;

    let claims = TestClaims {
        iss: None,
        sub: None,
        aud: None,
        exp: None,
        nbf: None,
        other: None,
    };

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_without_issuer() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.iss = None;

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_invalid_issuer() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.iss = Some("invalid issuer".to_string());

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_empty_issuer() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.iss = Some("".to_string());

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_too_long_issuer() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.iss = Some("a".repeat(1025));

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_without_subject() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.sub = None;

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_empty_subject() {
    let harness = TestHarness::new().await;

    let claims = harness.authentication.valid_claims("");

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_too_long_subject() {
    let harness = TestHarness::new().await;

    let claims = harness.authentication.valid_claims(&"a".repeat(1025));

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_without_audience() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.aud = None;

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_with_invalid_audience() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.aud = Some("invalid audience".to_string());

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_without_expiration() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.exp = None;

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_expired_token() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.exp = Some(
        SystemTime::now()
            .checked_sub(harness.settings.authentication.clock_skew + Duration::from_secs(1))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_reject_token_before_not_before() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.nbf = Some(
        SystemTime::now()
            .checked_add(harness.settings.authentication.clock_skew + Duration::from_secs(1))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_invalid_authentication_header(&harness, &header).await;
}
