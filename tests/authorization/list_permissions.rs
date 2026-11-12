use std::collections::HashSet;

use axum::http::header;
use serde::Deserialize;

use crate::common::{TestApplication, assert_unauthorized};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PermissionsResponse {
    permissions: Vec<String>,
}

#[tokio::test]
async fn should_list_permissions() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .get("/api/v1/permissions")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .await;

    response.assert_status_ok();
    assert_eq!(response.content_type(), "application/json");

    let body: PermissionsResponse = response.json();

    assert_eq!(body.permissions.len(), 6);

    let actual = body.permissions.into_iter().collect::<HashSet<_>>();
    let expected = HashSet::from([
        "roles:create".to_owned(),
        "roles:read".to_owned(),
        "roles:update".to_owned(),
        "roles:delete".to_owned(),
        "roles:assign".to_owned(),
        "roles:unassign".to_owned(),
    ]);

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn should_require_authentication_to_list_permissions() {
    let app = TestApplication::new().await;

    let response = app.server.get("/api/v1/permissions").await;

    assert_unauthorized(&response);
}
