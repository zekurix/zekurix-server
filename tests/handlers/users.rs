use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde::Deserialize;

use zekurix_server::user::UserId;

use crate::helpers::TestApp;

#[derive(Deserialize)]
struct UserResponse {
    id: UserId,
    username: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    code: String,
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
    let testapp = TestApp::new().await;
    let request = post_users("Alice");
    let response = testapp.request(request).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user: UserResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(user.username, "Alice");
}

#[tokio::test]
async fn should_get_user() {
    let testapp = TestApp::new().await;
    let request = post_users("Alice");
    let response = testapp.request(request).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user_post: UserResponse = serde_json::from_slice(&body).unwrap();
    let request = get_users(user_post.id);
    let response = testapp.request(request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let user_get: UserResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(user_get.id, user_post.id);
    assert_eq!(user_get.username, "Alice");
}

#[tokio::test]
async fn should_create_and_get_multiple_users() {
    let testapp = TestApp::new().await;
    let usernames = ["Alice", "Bob", "Charlie"];

    for username in usernames {
        let request = post_users(username);
        let response = testapp.request(request).await;
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let user_post: UserResponse = serde_json::from_slice(&body).unwrap();
        let request = get_users(user_post.id);
        let response = testapp.request(request).await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let user_get: UserResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(user_get.id, user_post.id);
        assert_eq!(user_get.username, *username);
    }
}

#[tokio::test]
async fn should_return_conflict_for_existing_user() {
    let testapp = TestApp::new().await;
    let request = post_users("Alice");
    let response = testapp.request(request).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let request = post_users("Alice");
    let response = testapp.request(request).await;

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let error: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.code, "USER_ALREADY_EXISTS");
}

#[tokio::test]
async fn should_return_not_found_for_invalid_user_id() {
    let testapp = TestApp::new().await;
    let request = get_users(UserId::new());
    let response = testapp.request(request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let error: ErrorResponse = serde_json::from_slice(&body).unwrap();
    assert_eq!(error.code, "USER_NOT_FOUND");
}

#[tokio::test]
async fn should_return_unprocessable_entity_for_invalid_payload() {
    let testapp = TestApp::new().await;
    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from("{}"))
        .unwrap();

    let response = testapp.request(request).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
