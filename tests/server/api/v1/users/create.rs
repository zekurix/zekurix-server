use axum::http::{StatusCode, header};
use serde_json::json;
use uuid::Uuid;

use crate::server::harness::TestHarness;
use crate::server::problem::{
    assert_http_forbidden, assert_http_unauthorized, assert_identity_invalid_issuer,
    assert_json_data_error, assert_user_already_exists,
};

use super::fixtures::{
    create_user, create_user_with_all_permissions_except, create_user_with_only_permission,
    create_user_without_permissions,
};
use super::models::UserResponse;
use super::requests::create_user as request_create_user;

#[tokio::test]
async fn should_create_user() {
    let harness = TestHarness::new().await;
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": "Alice",
        }
    });

    let response = request_create_user(&harness, body, "admin").await;

    response.assert_status(StatusCode::CREATED);

    let location = response
        .headers()
        .get(header::LOCATION)
        .expect("Location header should be present")
        .to_str()
        .unwrap();

    let user: UserResponse = response.json();

    Uuid::parse_str(&user.id).expect("user ID should be a valid UUID");
    assert_eq!(location, format!("/api/v1/users/{}", user.id));
}

#[tokio::test]
async fn should_reject_missing_identity() {
    let harness = TestHarness::new().await;

    let response = request_create_user(&harness, json!({}), "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_missing_identity_issuer() {
    let harness = TestHarness::new().await;

    let body = json!({
        "identity": {
            "subject": "Alice",
        }
    });

    let response = request_create_user(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_missing_identity_subject() {
    let harness = TestHarness::new().await;
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
        }
    });

    let response = request_create_user(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_empty_identity_issuer() {
    let harness = TestHarness::new().await;

    let body = json!({
        "identity": {
            "issuer": "",
            "subject": "Alice",
        }
    });

    let response = request_create_user(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_empty_identity_subject() {
    let harness = TestHarness::new().await;
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": "",
        }
    });

    let response = request_create_user(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_fields() {
    let harness = TestHarness::new().await;
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": "Alice",
        },
        "unknown_field": 42,
    });

    let response = request_create_user(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_identity_fields() {
    let harness = TestHarness::new().await;
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": "Alice",
            "unknown_field": 42,
        }
    });

    let response = request_create_user(&harness, body, "admin").await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_identity_with_invalid_issuer() {
    let harness = TestHarness::new().await;

    let body = json!({
        "identity": {
            "issuer": "https://invalid-issuer.example.com",
            "subject": "Alice",
        }
    });

    let response = request_create_user(&harness, body, "admin").await;

    assert_identity_invalid_issuer(&response);
}

#[tokio::test]
async fn should_return_conflict_for_existing_user() {
    let harness = TestHarness::new().await;

    create_user(&harness, "Alice").await;

    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": "Alice",
        }
    });

    let response = request_create_user(&harness, body, "admin").await;

    assert_user_already_exists(&response);
}

#[tokio::test]
async fn should_require_authentication_for_create_user() {
    let harness = TestHarness::new().await;
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": "Alice",
        }
    });

    let response = harness.server.post("/api/v1/users").json(&body).await;

    assert_http_unauthorized(&response);
}

#[tokio::test]
async fn should_create_user_for_user_with_only_users_create_permission() {
    let harness = TestHarness::new().await;
    let actor = create_user_with_only_permission(&harness, "users:create").await;
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": "Alice",
        }
    });

    let response = request_create_user(&harness, body, &actor).await;

    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn should_reject_user_creation_for_user_without_permissions() {
    let harness = TestHarness::new().await;
    let actor = create_user_without_permissions(&harness).await;
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": "Alice",
        }
    });

    let response = request_create_user(&harness, body, &actor).await;

    assert_http_forbidden(&response);
}

#[tokio::test]
async fn should_reject_user_creation_for_user_without_users_create_permission() {
    let harness = TestHarness::new().await;
    let actor = create_user_with_all_permissions_except(&harness, "users:create").await;
    let issuer = harness
        .settings
        .authentication
        .issuers
        .first()
        .expect("at least one issuer must be configured");

    let body = json!({
        "identity": {
            "issuer": issuer,
            "subject": "Alice",
        }
    });

    let response = request_create_user(&harness, body, &actor).await;

    assert_http_forbidden(&response);
}
