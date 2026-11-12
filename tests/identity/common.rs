use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::http::{StatusCode, header};
use axum_test::TestResponse;
use jsonwebtoken::{Header, encode};
use serde::Serialize;

use crate::common::{TestApplication, TestAuth, assert_problem, assert_unauthorized};

#[derive(Debug, Serialize)]
pub struct TestClaims {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<String>,
}

pub fn build_valid_claims(context: &TestAuth, subject: &str) -> TestClaims {
    let exp = SystemTime::now()
        .checked_add(Duration::from_mins(15))
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    TestClaims {
        iss: Some(context.issuer.clone()),
        sub: Some(subject.to_string()),
        aud: Some(context.audience.clone()),
        exp: Some(exp),
        nbf: None,
        other: None,
    }
}

pub fn build_valid_token(context: &TestAuth, claims: &TestClaims) -> String {
    let mut header = Header::new(context.algorithm);
    header.kid = Some(context.kid.clone());

    encode(&header, claims, &context.encoding_key).expect("failed to generate test JWT")
}

pub fn build_valid_header(token: &str) -> String {
    format!("Bearer {token}")
}

pub async fn assert_valid_auth_header(app: &TestApplication, header: &str) {
    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, header)
        .await;
    response.assert_status_ok();
    assert!(response.headers().get(header::WWW_AUTHENTICATE).is_none());
}

pub async fn assert_invalid_auth_header(app: &TestApplication, header: &str) {
    let response = app
        .server
        .get("/api/v1/users/me")
        .add_header(header::AUTHORIZATION, header)
        .await;
    assert_unauthorized(&response);
}

pub fn assert_invalid_issuer(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "https://api.zekurix.com/problems/identity/invalid-issuer",
    );
}
