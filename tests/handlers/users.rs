use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde::Deserialize;
use tower::ServiceExt;
use uuid::Uuid;

use zekurix_server::{application::Application, cli::Cli, router::build_router};

#[derive(Deserialize)]
struct UserResponse {
    id: Uuid,
    name: String,
}

fn post_users(name: &str) -> Request<Body> {
    let body = serde_json::json!({
        "name": name,
    });

    Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap()
}

fn get_users(id: Uuid) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri(format!("/users/{}", id))
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn should_create_user() {
    let cli = Cli::build().unwrap();
    let application = Application::build(&cli).unwrap();
    let router = build_router(application);

    let response = router.oneshot(post_users("Alice")).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: UserResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(user.name, "Alice");
}

#[tokio::test]
async fn should_get_user() {
    let cli = Cli::build().unwrap();
    let application = Application::build(&cli).unwrap();
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
    assert_eq!(user_get.name, "Alice");
}

#[tokio::test]
async fn should_create_and_get_multiple_users() {
    let cli = Cli::build().unwrap();
    let application = Application::build(&cli).unwrap();
    let router = build_router(application);

    let users = ["Alice", "Bob", "Charlie"];

    for name in users {
        let response = router.clone().oneshot(post_users(name)).await.unwrap();
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
        assert_eq!(user_get.name, *name);
    }
}

#[tokio::test]
async fn should_return_conflict_for_existing_user() {
    let cli = Cli::build().unwrap();
    let application = Application::build(&cli).unwrap();
    let router = build_router(application);

    let response = router.clone().oneshot(post_users("Alice")).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = router.clone().oneshot(post_users("Alice")).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn should_return_not_found_for_invalid_user_id() {
    let cli = Cli::build().unwrap();
    let application = Application::build(&cli).unwrap();
    let router = build_router(application);

    let response = router.oneshot(get_users(Uuid::new_v4())).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
