use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::http::header;
use jsonwebtoken::{Header, encode};

use crate::common::TestApplication;

use super::common::assert_valid_auth_header;
use super::common::build_valid_claims;
use super::common::build_valid_header;
use super::common::build_valid_token;

#[tokio::test]
async fn should_auth_with_helper() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;
    response.assert_status_ok();
}

#[tokio::test]
async fn should_auth_with_valid_header() {
    let app = TestApplication::new().await;

    let claims = build_valid_claims(app.auth.clone(), "Alice");
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_valid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_ignore_unknown_claims() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.other = Some("Other".to_string());
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_valid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_accept_recently_expired_token_within_leeway() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.exp = Some(
        SystemTime::now()
            .checked_sub(app.auth.clock_skew - Duration::from_secs(1))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_valid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_accept_token_with_valid_nbf() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.nbf = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_valid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_accept_token_with_valid_nbf_within_leeway() {
    let app = TestApplication::new().await;

    let mut claims = build_valid_claims(app.auth.clone(), "Alice");
    claims.nbf = Some(
        SystemTime::now()
            .checked_add(app.auth.clock_skew - Duration::from_secs(1))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
    );
    let token = build_valid_token(app.auth.clone(), claims);
    let header = build_valid_header(&token);

    assert_valid_auth_header(app, &header).await;
}

#[tokio::test]
async fn should_accept_token_without_kid() {
    let app = TestApplication::new().await;

    let claims = build_valid_claims(app.auth.clone(), "Alice");

    let token = encode(
        &Header::new(app.auth.algorithm),
        &claims,
        &app.auth.encoding_key,
    )
    .expect("failed to generate test JWT");

    let header = build_valid_header(&token);

    assert_valid_auth_header(app, &header).await;
}
