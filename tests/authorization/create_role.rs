use axum::http::{StatusCode, header};

use zekurix_server::authorization::RoleId;

use crate::common::{
    TestApplication, assert_forbidden, assert_json_data_error, assert_unauthorized,
};

use super::common::{
    RoleResponse, assert_duplicate_permission, assert_role_already_exists, create_role,
    create_user_with_all_permissions_except, create_user_with_only_permission,
    create_user_without_permissions,
};

#[tokio::test]
async fn should_create_role() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": ["roles:read"]
        }))
        .await;

    response.assert_status(StatusCode::CREATED);

    let location = response
        .headers()
        .get(header::LOCATION)
        .expect("Location header should be present")
        .to_str()
        .unwrap()
        .to_owned();

    let role: RoleResponse = response.json();

    assert_eq!(role.name, "user-manager");
    assert_eq!(role.permissions, vec!["roles:read"]);
    assert_ne!(role.id, RoleId::nil());
    assert_eq!(location, format!("/api/v1/roles/{}", role.id));
}

#[tokio::test]
async fn should_create_role_without_permissions() {
    let app = TestApplication::new().await;

    let role = create_role(&app, "empty-role", &[]).await;

    assert_eq!(role.name, "empty-role");
    assert!(role.permissions.is_empty());
}

#[tokio::test]
async fn should_create_role_with_multiple_permissions() {
    let app = TestApplication::new().await;

    let role = create_role(
        &app,
        "role-with-permissions",
        &["roles:create", "roles:read", "roles:update", "roles:delete"],
    )
    .await;

    assert_eq!(
        role.permissions,
        vec!["roles:create", "roles:read", "roles:update", "roles:delete"]
    );
}

#[tokio::test]
async fn should_reject_missing_name() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "permissions": []
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_missing_permissions() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager"
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_fields() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": [],
            "unknown_field": 42
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_permissions() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": ["unknown:permission"]
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_duplicate_permissions() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": ["roles:read", "roles:read"]
        }))
        .await;

    assert_duplicate_permission(&response);
}

#[tokio::test]
async fn should_return_conflict_for_existing_role() {
    let app = TestApplication::new().await;

    create_role(&app, "user-manager", &["roles:read"]).await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": ["roles:read"]
        }))
        .await;

    assert_role_already_exists(&response);
}

#[tokio::test]
async fn should_require_authentication_for_create_role() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/roles")
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": []
        }))
        .await;

    assert_unauthorized(&response);
}

#[tokio::test]
async fn should_reject_role_creation_for_user_without_permissions() {
    let app = TestApplication::new().await;
    let subject = "user-without-permissions";
    create_user_without_permissions(&app, subject).await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": []
        }))
        .await;

    assert_forbidden(&response);
}

#[tokio::test]
async fn should_create_role_for_user_with_only_roles_create_permission() {
    let app = TestApplication::new().await;
    let subject = "user-with-only-roles-create";
    create_user_with_only_permission(&app, subject, "roles:create").await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": []
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn should_reject_role_creation_for_user_without_roles_create_permission() {
    let app = TestApplication::new().await;
    let subject = "user-without-roles-create";
    create_user_with_all_permissions_except(&app, subject, "roles:create").await;

    let response = app
        .server
        .post("/api/v1/roles")
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .json(&serde_json::json!({
            "name": "user-manager",
            "permissions": []
        }))
        .await;

    assert_forbidden(&response);
}
