use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::timeout::TimeoutLayer;

use crate::handlers::health::health;

pub fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .layer((TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ),))
}
