use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

use zekurix_server::{application::Application, cli::Cli, router::build_router};

#[tokio::test]
async fn should_return_ok_for_health_endpoint() {
    let cli = Cli::build().unwrap();
    let application = Application::build(&cli).unwrap();
    let router = build_router(&application);

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

    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn should_return_not_found_for_unknown_route() {
    let cli = Cli::build().unwrap();
    let application = Application::build(&cli).unwrap();
    let router = build_router(&application);

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
