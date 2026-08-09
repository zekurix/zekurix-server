use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::timeout::TimeoutLayer;

use crate::application::Application;
use crate::handlers::health::health;

fn build_timeout_layer(application: &Application) -> TimeoutLayer {
    TimeoutLayer::with_status_code(
        StatusCode::REQUEST_TIMEOUT,
        Duration::from_secs(application.settings.server.timeout),
    )
}

pub fn build_router(application: &Application) -> Router {
    Router::new()
        .route("/health", get(health))
        .layer(build_timeout_layer(application))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;
    use crate::{application::Application, cli::Cli};

    #[tokio::test]
    async fn should_return_timeout_for_slow_route() {
        let cli = Cli::build().unwrap();
        let mut application = Application::build(&cli).unwrap();
        application.settings.server.timeout = 1;

        let slow_handler = || async {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        };

        let router = Router::new()
            .route("/slow", get(slow_handler))
            .layer(build_timeout_layer(&application));

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/slow")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }
}
