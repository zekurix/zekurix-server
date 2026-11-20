use std::collections::HashSet;

use crate::server::harness::TestHarness;
use crate::server::problem::assert_http_unauthorized;

use super::models::PermissionsResponse;
use super::requests::list_permissions;

#[tokio::test]
async fn should_list_permissions() {
    let harness = TestHarness::new().await;

    let response = list_permissions(&harness, "admin").await;

    response.assert_status_ok();
    assert_eq!(response.content_type(), "application/json");

    let body: PermissionsResponse = response.json();

    assert_eq!(body.permissions.len(), 9);

    let actual = body.permissions.into_iter().collect::<HashSet<_>>();
    let expected = HashSet::from([
        "roles:create".to_owned(),
        "roles:read".to_owned(),
        "roles:update".to_owned(),
        "roles:delete".to_owned(),
        "roles:assign".to_owned(),
        "roles:unassign".to_owned(),
        "users:create".to_owned(),
        "users:read".to_owned(),
        "users:delete".to_owned(),
    ]);

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn should_require_authentication_to_list_permissions() {
    let harness = TestHarness::new().await;

    let response = harness.server.get("/api/v1/permissions").await;

    assert_http_unauthorized(&response);
}
