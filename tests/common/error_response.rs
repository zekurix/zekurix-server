use axum::http::{StatusCode, header};
use axum_test::TestResponse;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorResponse {
    #[serde(rename = "type")]
    pub r#type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub _instance: Option<String>,
}

pub fn assert_problem(response: &TestResponse, expected_status: StatusCode, expected_type: &str) {
    response.assert_status(expected_status);
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ErrorResponse = response.json();
    assert_eq!(body.r#type, expected_type);
    assert!(!body.title.is_empty());
    assert_eq!(body.status, expected_status.as_u16());
    assert!(!body.detail.is_empty());
}

pub fn assert_unauthorized(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::UNAUTHORIZED,
        "https://api.zekurix.com/problems/http/unauthorized",
    );

    let www_authenticate = response
        .headers()
        .get(header::WWW_AUTHENTICATE)
        .expect("WWW-Authenticate header should be present");
    assert_eq!(www_authenticate.to_str().unwrap(), "Bearer");
}

pub fn assert_json_data_error(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "https://api.zekurix.com/problems/json/data-error",
    );
}

pub fn assert_invalid_query(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::BAD_REQUEST,
        "https://api.zekurix.com/problems/query/invalid",
    );
}
