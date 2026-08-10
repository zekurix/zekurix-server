use std::sync::Arc;
use std::time::Duration;

use axum::{Router, http::StatusCode, routing::{get, post}};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

use crate::application::Application;
use crate::handlers::health::health;
use crate::handlers::users::{create_user, get_user};

fn build_timeout_layer(timeout: u64) -> TimeoutLayer {
    TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(timeout))
}

pub fn build_router(application: Arc<Application>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/users", post(create_user))
        .route("/users/{id}", get(get_user))
        .layer((
            TraceLayer::new_for_http(),
            build_timeout_layer(application.settings.server.timeout),
        ))
        .with_state(application)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn should_return_timeout_for_slow_route() {
        let slow_handler = || async {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        };

        let router = Router::new()
            .route("/slow", get(slow_handler))
            .layer(build_timeout_layer(1));

        let response = router
            .oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }
}
