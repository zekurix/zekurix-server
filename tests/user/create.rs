use axum::http::{StatusCode, header};

use zekurix_server::user::UserId;

use crate::common::{TestApplication, assert_json_data_error, assert_unauthorized};
use crate::identity::assert_invalid_issuer;

use super::common::{UserResponse, assert_user_already_exists};

pub async fn create_user(app: &TestApplication, subject: &str) -> UserResponse {
    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": app.auth.issuer.clone(),
                "subject": subject
            }
        }))
        .await;
    response.assert_status(StatusCode::CREATED);

    response.json()
}

pub async fn create_users(app: &TestApplication, subjects: &[&str]) -> Vec<UserId> {
    assert_eq!(
        subjects.first(),
        Some(&"admin"),
        "admin must be the first fixture user"
    );

    let mut users = Vec::with_capacity(subjects.len());

    for subject in subjects {
        let user = if *subject == "admin" {
            let response = app
                .server
                .get("/api/v1/users/me")
                .add_header(header::AUTHORIZATION, app.auth.header("admin"))
                .await;

            response.assert_status_ok();
            response.json::<UserResponse>()
        } else {
            create_user(app, subject).await
        };

        users.push(user.id);
    }

    // Match the repository's ORDER BY id.
    users.sort();
    users
}

#[tokio::test]
async fn should_create_user() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": app.auth.issuer,
                "subject": "Alice"
            }
        }))
        .await;
    response.assert_status(StatusCode::CREATED);

    let location = response
        .headers()
        .get(header::LOCATION)
        .expect("Location header should be present")
        .to_str()
        .unwrap();
    let user: UserResponse = response.json();

    assert_eq!(location, format!("/api/v1/users/{}", user.id));
}

#[tokio::test]
async fn should_reject_missing_identity() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({}))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_missing_identity_issuer() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "subject": "Alice"
            }
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_missing_identity_subject() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": app.auth.issuer,
            }
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_empty_identity_issuer() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": "",
                "subject": "Alice"
            }
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_empty_identity_subject() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": app.auth.issuer,
                "subject": ""
            }
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_fields() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": app.auth.issuer,
                "subject": "Alice"
            },
            "unknown_field": 42
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_reject_unknown_identity_fields() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": app.auth.issuer,
                "subject": "Alice",
                "unknown_field": 42
            }
        }))
        .await;

    assert_json_data_error(&response);
}

#[tokio::test]
async fn should_return_conflict_for_existing_user() {
    let app = TestApplication::new().await;

    create_user(&app, "Alice").await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": app.auth.issuer,
                "subject": "Alice"
            }
        }))
        .await;

    assert_user_already_exists(&response);
}

#[tokio::test]
async fn should_require_authentication_for_create_user() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .json(&serde_json::json!({
            "identity": {
                "issuer": app.auth.issuer,
                "subject": "Alice"
            }
        }))
        .await;
    assert_unauthorized(&response);
}

#[tokio::test]
async fn should_reject_identity_with_invalid_issuer() {
    let app = TestApplication::new().await;

    let response = app
        .server
        .post("/api/v1/users")
        .add_header(header::AUTHORIZATION, app.auth.header("admin"))
        .json(&serde_json::json!({
            "identity": {
                "issuer": "https://invalid-issuer.example.com",
                "subject": "Alice"
            }
        }))
        .await;

    assert_invalid_issuer(&response);
}
