use axum::Json;
use serde::Serialize;

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

pub async fn health() -> Json<HealthResponse> {
    Json(build_health_response())
}
