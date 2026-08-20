use std::sync::Arc;
use std::time::Duration;

use axum::{Router, http::StatusCode};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::application::Application;
use crate::health;
use crate::user;

fn timeout_layer(timeout: u64) -> TimeoutLayer {
    TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(timeout))
}

pub fn router(application: Arc<Application>) -> Router {
    Router::new()
        .nest("/health", health::router())
        .nest("/users", user::router())
        .layer((
            TraceLayer::new_for_http(),
            timeout_layer(application.settings.server.timeout),
        ))
        .with_state(application)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    #[tokio::test]
    async fn should_return_timeout_for_slow_route() {
        let slow_handler = || async {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        };

        let router = Router::new()
            .route("/slow", get(slow_handler))
            .layer(timeout_layer(1));

        let response = router
            .oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }
}
