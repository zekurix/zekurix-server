use axum::http::header;

use crate::server::harness::TestHarness;
use crate::server::problem::assert_path_rejection;

#[tokio::test]
async fn should_return_problem_details_for_path_rejection() {
    let harness = TestHarness::new().await;

    let response = harness
        .server
        .get("/api/v1/users/invalid-uuid")
        .add_header(
            header::AUTHORIZATION,
            harness.authentication.header("admin"),
        )
        .await;

    assert_path_rejection(&response);
}
