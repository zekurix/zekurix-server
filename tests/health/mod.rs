use serde::Deserialize;

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
