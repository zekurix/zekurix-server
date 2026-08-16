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

#[derive(Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[tokio::test]
async fn should_return_health_response() {
    dotenv().ok();
    let mut settings = Settings::default();
    settings.database.username = Some("postgres".to_string());
    let application = Application::new(settings).await.unwrap();
    let router = build_router(application);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let health: HealthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(health.status, "ok");
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn should_return_not_found_for_unknown_route() {
    dotenv().ok();
    let mut settings = Settings::default();
    settings.database.username = Some("postgres".to_string());
    let application = Application::new(settings).await.unwrap();
    let router = build_router(application);

    let response = router
        .oneshot(
            Request::builder()
                .uri("/foobar")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
