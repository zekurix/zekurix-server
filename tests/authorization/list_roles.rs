use axum::http::header;
use axum_test::TestResponse;

use zekurix_server::authorization::RoleId;

use crate::common::{TestApplication, assert_forbidden, assert_invalid_query, assert_unauthorized};

use super::common::{
    RolesResponse, create_role, create_user_with_all_permissions_except,
    create_user_with_only_permission, create_user_without_permissions,
};

async fn list_roles(app: &TestApplication, subject: &str, query: Option<&str>) -> TestResponse {
    let uri = match query {
        Some(query) => format!("/api/v1/roles?{query}"),
        None => "/api/v1/roles".to_owned(),
    };

    app.server
        .get(&uri)
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .await
}

fn role_ids(response: TestResponse) -> Vec<RoleId> {
    response.assert_status_ok();
    assert_eq!(response.content_type(), "application/json");

    let body: RolesResponse = response.json();
    body.roles.into_iter().map(|role| role.id).collect()
}

fn assert_role_ids(response: TestResponse, expected: &[RoleId]) {
    let actual = role_ids(response);

    assert_eq!(actual, expected);
}

async fn existing_role_ids(app: &TestApplication) -> Vec<RoleId> {
    role_ids(list_roles(app, "admin", None).await)
}

async fn create_roles(app: &TestApplication, names: &[&str]) -> Vec<RoleId> {
    let mut ids = Vec::with_capacity(names.len());
    for name in names {
        ids.push(create_role(app, name, &[]).await.id);
    }

    // The repository returns roles ordered by their ID.
    ids.sort();
    ids
}

#[tokio::test]
async fn should_list_roles_with_default_pagination() {
    let app = TestApplication::new().await;
    let mut expected = existing_role_ids(&app).await;
    expected.extend(create_roles(&app, &["reader", "editor", "manager"]).await);
    expected.sort();

    let response = list_roles(&app, "admin", None).await;

    assert_role_ids(response, &expected);
}

#[tokio::test]
async fn should_apply_limit_and_offset() {
    let app = TestApplication::new().await;
    let mut roles = existing_role_ids(&app).await;
    roles.extend(create_roles(&app, &["one", "two", "three", "four"]).await);
    roles.sort();

    let response = list_roles(&app, "admin", Some("limit=2&offset=1")).await;

    assert_role_ids(response, &roles[1..3]);
}

#[tokio::test]
async fn should_return_an_empty_page_when_offset_exceeds_role_count() {
    let app = TestApplication::new().await;
    create_roles(&app, &["one", "two"]).await;

    let response = list_roles(&app, "admin", Some("limit=2&offset=100")).await;

    assert_role_ids(response, &[]);
}

#[tokio::test]
async fn should_reject_invalid_pagination() {
    let app = TestApplication::new().await;

    for query in [
        "limit=0",
        "limit=101",
        "limit=-1",
        "limit=invalid",
        "offset=-1",
    ] {
        let response = list_roles(&app, "admin", Some(query)).await;
        assert_invalid_query(&response);
    }
}

#[tokio::test]
async fn should_require_authentication_to_list_roles() {
    let app = TestApplication::new().await;

    let response = app.server.get("/api/v1/roles").await;

    assert_unauthorized(&response);
}

#[tokio::test]
async fn should_list_roles_for_user_with_only_roles_read_permission() {
    let app = TestApplication::new().await;
    let subject = "user-with-only-roles-read";
    create_user_with_only_permission(&app, subject, "roles:read").await;

    let mut expected = existing_role_ids(&app).await;
    expected.extend(create_roles(&app, &["reader", "editor"]).await);
    expected.sort();

    let response = list_roles(&app, subject, None).await;

    assert_role_ids(response, &expected);
}

#[tokio::test]
async fn should_reject_role_listing_for_user_without_permissions() {
    let app = TestApplication::new().await;
    let subject = "user-without-permissions";
    create_user_without_permissions(&app, subject).await;

    let response = list_roles(&app, subject, None).await;

    assert_forbidden(&response);
}

#[tokio::test]
async fn should_reject_role_listing_for_user_without_roles_read_permission() {
    let app = TestApplication::new().await;
    let subject = "user-without-roles-read";
    create_user_with_all_permissions_except(&app, subject, "roles:read").await;

    let response = list_roles(&app, subject, None).await;

    assert_forbidden(&response);
}
