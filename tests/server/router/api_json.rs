use axum::{body::Bytes, http::header};

use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_json_bytes_rejection, assert_json_data_error, assert_json_missing_content_type,
    assert_json_syntax_error,
};

#[tokio::test]
async fn should_return_problem_details_for_json_data_error() {
    let harness = TestHarness::new().await;

    let response = harness
        .server
        .post("/api/v1/users")
        .add_header(
            header::AUTHORIZATION,
            harness.authentication.header("admin"),
        )
        .json(&serde_json::json!(42))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_return_problem_details_for_json_syntax_error() {
    let harness = TestHarness::new().await;

    let response = harness
        .server
        .post("/api/v1/users")
        .add_header(
            header::AUTHORIZATION,
            harness.authentication.header("admin"),
        )
        .bytes(Bytes::from("not a json"))
        .add_header("content-type", "application/json")
        .await;

    assert_json_syntax_error(&response);
}

#[tokio::test]
async fn should_return_problem_details_for_missing_json_content_type() {
    let harness = TestHarness::new().await;

    let response = harness
        .server
        .post("/api/v1/users")
        .add_header(
            header::AUTHORIZATION,
            harness.authentication.header("admin"),
        )
        .bytes(Bytes::from(
            r#"{"identity": {"issuer": "https://auth.example.com/my-issuer", "subject": "Alice"}}"#,
        ))
        .await;

    assert_json_missing_content_type(&response);
}

#[tokio::test]
async fn should_return_problem_details_for_json_bytes_rejection() {
    let harness = TestHarness::new().await;

    let response = harness
        .server
        .post("/api/v1/users")
        .add_header(
            header::AUTHORIZATION,
            harness.authentication.header("admin"),
        )
        .bytes(Bytes::from(vec![b'a'; 3_000_000]))
        .add_header("content-type", "application/json")
        .await;

    assert_json_bytes_rejection(&response);
}
