use crate::server::harness::TestHarness;

#[tokio::test]
async fn should_return_openapi_spec() {
    let harness = TestHarness::new().await;

    let response = harness.server.get("/openapi.json").await;
    response.assert_status_ok();

    let body: serde_json::Value = serde_json::from_str(&response.text()).unwrap();

    assert_eq!(body["openapi"], "3.1.0");
    assert!(body["paths"].is_object());

    assert_eq!(
        body["components"]["schemas"]["Identity"]["properties"]["issuer"]["enum"],
        serde_json::json!(&harness.settings.authentication.issuers)
    );
}
