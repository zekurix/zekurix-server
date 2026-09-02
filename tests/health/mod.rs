use serde::Deserialize;
use uuid::Uuid;

use crate::common::TestApplication;

#[derive(Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[tokio::test]
async fn should_return_health_response() {
    let app = TestApplication::new().await;

    let response = app.server.get("/health").await;
    response.assert_status_ok();

    let health: HealthResponse = response.json();
    assert_eq!(health.status, "ok");
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn should_return_not_found_for_unknown_route() {
    let app = TestApplication::new().await;

    let response = app.server.get("/foobar").await;
    response.assert_status_not_found();
}

#[tokio::test]
async fn should_echo_request_id_when_provided() {
    let app = TestApplication::new().await;
    let request_id_request = Uuid::now_v7().to_string();

    let response = app
        .server
        .get("/health")
        .add_header("X-Request-Id", request_id_request.clone())
        .await;
    response.assert_status_ok();

    let request_id_response = response
        .headers()
        .get("X-Request-Id")
        .unwrap()
        .to_str()
        .unwrap();

    assert_eq!(request_id_response, request_id_request);
}

#[tokio::test]
async fn should_generate_request_id_when_provided_id_is_invalid() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .get("/health")
        .add_header("X-Request-Id", "not-a-uuid")
        .await;

    response.assert_status_ok();

    let request_id = response
        .headers()
        .get("X-Request-Id")
        .unwrap()
        .to_str()
        .unwrap();

    assert_ne!(request_id, "not-a-uuid");
    assert!(
        Uuid::parse_str(request_id).is_ok(),
        "response request ID should be a valid UUID"
    );
}

#[tokio::test]
async fn should_generate_request_id_when_missing() {
    let app = TestApplication::new().await;

    let response = app.server.get("/health").await;
    response.assert_status_ok();

    let request_id = response
        .headers()
        .get("X-Request-Id")
        .unwrap()
        .to_str()
        .unwrap();

    assert!(!request_id.is_empty());
    uuid::Uuid::parse_str(request_id).expect("generated request id should be a valid UUID");
}
