use axum::http::{StatusCode, header};
use axum_test::TestResponse;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProblemDetails {
    #[serde(rename = "type")]
    pub(super) r#type: String,
    pub(super) title: String,
    pub(super) status: u16,
    pub(super) detail: String,
    pub(super) _instance: Option<String>,
}

pub(super) fn assert_problem(
    response: &TestResponse,
    expected_status: StatusCode,
    expected_type: &str,
) {
    response.assert_status(expected_status);
    assert_eq!(response.content_type(), "application/problem+json");

    let body: ProblemDetails = response.json();
    assert_eq!(body.r#type, expected_type);
    assert!(!body.title.is_empty());
    assert_eq!(body.status, expected_status.as_u16());
    assert!(!body.detail.is_empty());
}

pub(super) fn assert_http_not_found(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::NOT_FOUND,
        "https://api.zekurix.com/problems/http/not-found",
    );
}

pub(super) fn assert_http_method_not_allowed(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::METHOD_NOT_ALLOWED,
        "https://api.zekurix.com/problems/http/method-not-allowed",
    );
}

pub(super) fn assert_http_unauthorized(response: &TestResponse) {
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

pub(super) fn assert_http_forbidden(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::FORBIDDEN,
        "https://api.zekurix.com/problems/http/forbidden",
    );
}

pub(super) fn assert_json_syntax_error(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::BAD_REQUEST,
        "https://api.zekurix.com/problems/json/syntax-error",
    );
}

pub(super) fn assert_json_data_error(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "https://api.zekurix.com/problems/json/data-error",
    );
}

pub(super) fn assert_json_missing_content_type(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "https://api.zekurix.com/problems/json/missing-content-type",
    );
}

pub(super) fn assert_json_bytes_rejection(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::PAYLOAD_TOO_LARGE,
        "https://api.zekurix.com/problems/json/bytes-rejection",
    );
}

pub(super) fn assert_path_rejection(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::BAD_REQUEST,
        "https://api.zekurix.com/problems/path/rejection",
    );
}

pub(super) fn assert_query_invalid(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::BAD_REQUEST,
        "https://api.zekurix.com/problems/query/invalid",
    );
}

pub(super) fn assert_identity_invalid_issuer(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "https://api.zekurix.com/problems/identity/invalid-issuer",
    );
}

pub(super) fn assert_role_not_found(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::NOT_FOUND,
        "https://api.zekurix.com/problems/role/not-found",
    );
}

pub(super) fn assert_role_already_exists(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::CONFLICT,
        "https://api.zekurix.com/problems/role/already-exists",
    );
}

pub(super) fn assert_permission_duplicate(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::UNPROCESSABLE_ENTITY,
        "https://api.zekurix.com/problems/permission/duplicate",
    );
}

pub(super) fn assert_user_not_found(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::NOT_FOUND,
        "https://api.zekurix.com/problems/user/not-found",
    );
}

pub(super) fn assert_user_already_exists(response: &TestResponse) {
    assert_problem(
        response,
        StatusCode::CONFLICT,
        "https://api.zekurix.com/problems/user/already-exists",
    );
}
