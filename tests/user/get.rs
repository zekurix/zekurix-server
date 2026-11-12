use axum::http::header;
use axum_test::TestResponse;

use zekurix_server::user::UserId;

use crate::common::{TestApplication, assert_unauthorized};

use super::common::{UserResponse, assert_user_not_found};
use super::create::create_user;

pub async fn get_user(app: &TestApplication, user_id: UserId, subject: &str) -> TestResponse {
    app.server
        .get(&format!("/api/v1/users/{user_id}"))
        .add_header(header::AUTHORIZATION, app.auth.header(subject))
        .await
}

#[tokio::test]
async fn should_get_user() {
    let app = TestApplication::new().await;

    let created = create_user(&app, "Alice").await;

    let response = get_user(&app, created.id, "Alice").await;
    response.assert_status_ok();

    let fetched: UserResponse = response.json();
    assert_eq!(fetched, created);
}

#[tokio::test]
async fn should_create_and_get_multiple_users() {
    let app = TestApplication::new().await;
    let subjects = ["Alice", "Bob", "Charlie"];

    for subject in subjects {
        let created = create_user(&app, subject).await;

        let response = get_user(&app, created.id, subject).await;
        response.assert_status_ok();

        let fetched: UserResponse = response.json();
        assert_eq!(fetched, created);
    }
}

#[tokio::test]
async fn should_return_not_found_for_invalid_user_id() {
    let app = TestApplication::new().await;

    let user_id = UserId::new();
    let response = app
        .server
        .get(&format!("/api/v1/users/{}", user_id))
        .add_header(header::AUTHORIZATION, app.auth.header("Alice"))
        .await;

    assert_user_not_found(&response);
}

#[tokio::test]
async fn should_require_authentication_to_get_user() {
    let app = TestApplication::new().await;

    let user = create_user(&app, "Alice").await;

    let response = app.server.get(&format!("/api/v1/users/{}", user.id)).await;
    assert_unauthorized(&response);
}
