use serde::Deserialize;

use crate::server::harness::TestHarness;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HealthResponse {
    status: String,
    version: String,
}

#[tokio::test]
async fn should_return_health_response() {
    let harness = TestHarness::new().await;

    let response = harness.server.get("/health").await;
    response.assert_status_ok();

    let health: HealthResponse = response.json();

    assert_eq!(health.status, "ok");
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
}
