use crate::common::TestApplication;

#[tokio::test]
async fn should_return_openapi_spec() {
    let app = TestApplication::new().await;

    let response = app.server.get("/openapi.json").await;
    response.assert_status_ok();

    let body: serde_json::Value = serde_json::from_str(&response.text()).unwrap();
    assert_eq!(body["openapi"], "3.1.0");
    assert!(body["paths"].is_object());
}
