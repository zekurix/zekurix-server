use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// Response returned by the health check endpoint.
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// Current health status of the application.
    #[schema(example = "ok")]
    status: &'static str,

    /// Current application version.
    #[schema(example = "1.0.0")]
    version: &'static str,
}

fn build_health_response() -> HealthResponse {
    HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    }
}

/// Health check endpoint.
///
/// Returns the current application status and version.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (
            status = 200,
            description = "Current application status and version",
            body = HealthResponse
        )
    )
)]
pub async fn get_health() -> Json<HealthResponse> {
    Json(build_health_response())
}
