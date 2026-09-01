use axum::Json;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Ok,
}

/// Response returned by the health check endpoint.
#[derive(Serialize)]
pub struct HealthResponse {
    status: HealthStatus,
    version: &'static str,
}

fn build_health_response() -> HealthResponse {
    HealthResponse {
        status: HealthStatus::Ok,
        version: env!("CARGO_PKG_VERSION"),
    }
}

pub async fn get_health() -> Json<HealthResponse> {
    Json(build_health_response())
}
