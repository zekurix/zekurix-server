use crate::server::harness::TestHarness;
use crate::server::problem::{assert_http_method_not_allowed, assert_http_not_found};

#[tokio::test]
async fn should_return_not_found_for_unknown_route() {
    let harness = TestHarness::new().await;

    let response = harness.server.get("/foobar").await;

    assert_http_not_found(&response);
}

#[tokio::test]
async fn should_return_method_not_allowed_for_invalid_method() {
    let harness = TestHarness::new().await;

    let response = harness.server.post("/health").await;

    assert_http_method_not_allowed(&response);
}
