use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use dotenv::dotenv;
use http_body_util::BodyExt;
use serde::Deserialize;
use tower::ServiceExt;

use zekurix_server::application::Application;
use zekurix_server::router::build_router;
use zekurix_server::settings::Settings;
use zekurix_server::user::UserId;

#[derive(Deserialize)]
struct UserResponse {
    id: UserId,
    username: String,
}

fn post_users(username: &str) -> Request<Body> {
    let body = serde_json::json!({
        "username": username,
    });

    Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

fn get_users(id: UserId) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!("/users/{}", id))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn should_create_user() {
    dotenv().ok();
    let mut settings = Settings::default();
    settings.database.username = Some("postgres".to_string());
    let application = Application::new(settings).await.unwrap();
    let router = build_router(application);

    let response = router.oneshot(post_users("Alice")).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: UserResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(user.username, "Alice");
}

#[tokio::test]
async fn should_get_user() {
    dotenv().ok();
    let mut settings = Settings::default();
    settings.database.username = Some("postgres".to_string());
    let application = Application::new(settings).await.unwrap();
    let router = build_router(application);

    let response = router.clone().oneshot(post_users("Alice")).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user_post: UserResponse = serde_json::from_slice(&body).unwrap();
    let response = router
        .clone()
        .oneshot(get_users(user_post.id))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user_get: UserResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(user_get.id, user_post.id);
    assert_eq!(user_get.username, "Alice");
}

#[tokio::test]
async fn should_create_and_get_multiple_users() {
    dotenv().ok();
    let mut settings = Settings::default();
    settings.database.username = Some("postgres".to_string());
    let application = Application::new(settings).await.unwrap();
    let router = build_router(application);

    let usernames = ["Alice", "Bob", "Charlie"];

    for username in usernames {
        let response = router.clone().oneshot(post_users(username)).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let user_post: UserResponse = serde_json::from_slice(&body).unwrap();
        let response = router
            .clone()
            .oneshot(get_users(user_post.id))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let user_get: UserResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(user_get.id, user_post.id);
        assert_eq!(user_get.username, *username);
    }
}

#[tokio::test]
async fn should_return_conflict_for_existing_user() {
    dotenv().ok();
    let mut settings = Settings::default();
    settings.database.username = Some("postgres".to_string());
    let application = Application::new(settings).await.unwrap();
    let router = build_router(application);

    let response = router.clone().oneshot(post_users("Alice")).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = router.clone().oneshot(post_users("Alice")).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn should_return_not_found_for_invalid_user_id() {
    dotenv().ok();
    let mut settings = Settings::default();
    settings.database.username = Some("postgres".to_string());
    let application = Application::new(settings).await.unwrap();
    let router = build_router(application);

    let response = router.oneshot(get_users(UserId::new())).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
