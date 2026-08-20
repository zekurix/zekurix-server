use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde::Deserialize;

use crate::common::TestApp;

#[derive(Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[tokio::test]
async fn should_return_health_response() {
    let testapp = TestApp::new().await;
    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let response = testapp.request(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let health: HealthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(health.status, "ok");
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn should_return_not_found_for_unknown_route() {
    let testapp = TestApp::new().await;
    let request = Request::builder()
        .uri("/foobar")
        .body(Body::empty())
        .unwrap();
    let response = testapp.request(request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
