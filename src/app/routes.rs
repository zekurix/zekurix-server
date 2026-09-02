use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use tower::ServiceBuilder;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use uuid::Uuid;

use crate::health;
use crate::openapi;
use crate::user;

use super::Application;

pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .filter(|value| {
            value
                .to_str()
                .ok()
                .and_then(|value| Uuid::parse_str(value).ok())
                .is_some()
        })
        .cloned()
        .unwrap_or_else(|| {
            HeaderValue::from_str(&Uuid::now_v7().to_string())
                .expect("UUID is always a valid header value")
        });
    request.extensions_mut().insert(request_id.clone());

    let mut response = next.run(request).await;

    response.headers_mut().insert("x-request-id", request_id);
    response
}

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
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(request_id_middleware))
                .layer(TraceLayer::new_for_http())
                .layer(timeout_layer(application.settings.server.timeout)),
        )
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
