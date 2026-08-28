use std::sync::Arc;
use std::time::Duration;

use axum::{Router, http::StatusCode};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::health;
use crate::openapi;
use crate::user;

use super::Application;

fn timeout_layer(timeout: Duration) -> TimeoutLayer {
    TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, timeout)
}

fn api_v1_router() -> Router<Arc<Application>> {
    Router::new().nest("/users", user::router())
}

pub fn router(application: Arc<Application>) -> Router {
    Router::new()
        .merge(openapi::router())
        .nest("/health", health::router())
        .nest("/api/v1", api_v1_router())
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
            tokio::time::sleep(Duration::from_secs(2)).await;
        };

        let router = Router::new()
            .route("/slow", get(slow_handler))
            .layer(timeout_layer(Duration::from_secs(1)));

        let response = router
            .oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }
}
