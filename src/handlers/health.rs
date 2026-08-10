use axum::{Json, http::StatusCode};
use serde::Serialize;
use tracing::debug;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

fn build_health_response() -> HealthResponse {
    HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    }
}

pub async fn health() -> (StatusCode, Json<HealthResponse>) {
    let status = StatusCode::OK;
    let response = Json(build_health_response());

    debug!(status = ?status, response = ?response);
    (status, response)
}
