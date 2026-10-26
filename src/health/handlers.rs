use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::Application;

use super::{HealthCheck, HealthStatus};

#[derive(Serialize)]
pub struct HealthResponse {
    status: HealthStatus,
    version: &'static str,
}

async fn build_health_response(application: Arc<Application>) -> HealthResponse {
    let status = application.services.health().await;
    let version = env!("CARGO_PKG_VERSION");

    HealthResponse { status, version }
}

fn status_code(status: HealthStatus) -> StatusCode {
    match status {
        HealthStatus::Ok | HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub async fn get_health(
    State(application): State<Arc<Application>>,
) -> (StatusCode, Json<HealthResponse>) {
    let response = build_health_response(application).await;
    let status = status_code(response.status);

    (status, Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_code_returns_ok_when_healthy() {
        assert_eq!(status_code(HealthStatus::Ok), StatusCode::OK);
    }

    #[test]
    fn status_code_returns_ok_when_degraded() {
        assert_eq!(status_code(HealthStatus::Degraded), StatusCode::OK);
    }

    #[test]
    fn status_code_returns_service_unavailable_when_unhealthy() {
        assert_eq!(
            status_code(HealthStatus::Unhealthy),
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
