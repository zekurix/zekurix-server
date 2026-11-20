use axum::http::StatusCode;
use serde_json::json;

use crate::server::api::v1::permissions::{
    models::PermissionsResponse, requests::list_permissions,
};
use crate::server::api::v1::roles::fixtures::create_role;
use crate::server::harness::TestHarness;

use super::models::UserResponse;
use super::requests::{assign_role, create_user as request_create_user, get_current_user};

pub(super) async fn create_user(harness: &TestHarness, subject: &str) -> UserResponse {
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": subject,
        }
    });

    let response = request_create_user(harness, body, "admin").await;

    response.assert_status(StatusCode::CREATED);
    response.json()
}

pub(super) async fn create_users(harness: &TestHarness, subjects: &[&str]) -> Vec<String> {
    assert_eq!(
        subjects.first(),
        Some(&"admin"),
        "admin must be the first fixture user"
    );

    let mut user_ids = Vec::with_capacity(subjects.len());

    for subject in subjects {
        let user = if *subject == "admin" {
            let response = get_current_user(harness, "admin").await;

            response.assert_status_ok();
            response.json::<UserResponse>()
        } else {
            create_user(harness, subject).await
        };

        user_ids.push(user.id);
    }

    // Match the API's ordering by user ID.
    user_ids.sort();
    user_ids
}

pub(crate) async fn create_user_without_permissions(harness: &TestHarness) -> String {
    let actor = "user-without-permissions".to_string();

    create_user(harness, &actor).await;

    actor
}

pub(crate) async fn create_user_with_only_permission(
    harness: &TestHarness,
    permission: &str,
) -> String {
    let actor = format!("user-with-only-{permission}");

    let user = create_user(harness, &actor).await;

    let role_name = format!("role-with-only-{permission}");
    let role = create_role(harness, &role_name, &[permission]).await;

    assign_role(harness, &user.id, &role.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    actor
}

pub(crate) async fn create_user_with_all_permissions_except(
    harness: &TestHarness,
    excluded_permission: &str,
) -> String {
    let actor = format!("user-without-{excluded_permission}");

    let user = create_user(harness, &actor).await;

    let response = list_permissions(harness, "admin").await;
    response.assert_status_ok();

    let all_permissions = response.json::<PermissionsResponse>().permissions;

    let permissions = all_permissions
        .iter()
        .map(String::as_str)
        .filter(|permission| *permission != excluded_permission)
        .collect::<Vec<_>>();

    assert!(
        permissions.len() < all_permissions.len(),
        "permission {excluded_permission} must exist"
    );

    let role_name = format!("role-with-all-permissions-except-{excluded_permission}");
    let role = create_role(harness, &role_name, &permissions).await;

    assign_role(harness, &user.id, &role.id, "admin")
        .await
        .assert_status(StatusCode::NO_CONTENT);

    actor
}
