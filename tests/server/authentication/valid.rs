use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::http::header;
use jsonwebtoken::Header;

use crate::server::harness::TestHarness;

use super::assertions::assert_valid_authentication_header;

#[tokio::test]
async fn should_authenticate_with_helper() {
    let harness = TestHarness::new().await;

    let response = harness
        .server
        .get("/api/v1/users/me")
        .add_header(
            header::AUTHORIZATION,
            harness.authentication.header("Alice"),
        )
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn should_authenticate_with_valid_header() {
    let harness = TestHarness::new().await;

    let claims = harness.authentication.valid_claims("Alice");
    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_valid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_ignore_unknown_claims() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.other = Some("Other".to_string());

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_valid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_accept_recently_expired_token_within_leeway() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.exp = Some(
        SystemTime::now()
            .checked_sub(harness.settings.authentication.clock_skew - Duration::from_secs(1))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_valid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_accept_token_with_valid_nbf() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.nbf = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_valid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_accept_token_with_valid_nbf_within_leeway() {
    let harness = TestHarness::new().await;

    let mut claims = harness.authentication.valid_claims("Alice");
    claims.nbf = Some(
        SystemTime::now()
            .checked_add(harness.settings.authentication.clock_skew - Duration::from_secs(1))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );

    let token = harness.authentication.token_with_claims(&claims);
    let header = harness.authentication.header_from_token(&token);

    assert_valid_authentication_header(&harness, &header).await;
}

#[tokio::test]
async fn should_accept_token_without_kid() {
    let harness = TestHarness::new().await;

    let claims = harness.authentication.valid_claims("Alice");

    let jwt_header = Header::new(harness.authentication.algorithm());
    let token = harness
        .authentication
        .token_with_header(jwt_header, &claims);

    let header = harness.authentication.header_from_token(&token);

    assert_valid_authentication_header(&harness, &header).await;
}
