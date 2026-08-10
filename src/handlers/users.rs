use std::sync::Arc;

use axum::{extract::{Path, State}, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use crate::application::Application;

#[derive(Deserialize)]
pub struct UserParams {
    name: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    name: String,
}

pub async fn create_user(
    State(_application): State<Arc<Application>>,
    Json(params): Json<UserParams>,
) -> (StatusCode, Json<UserResponse>) {
    let status = StatusCode::CREATED;
    let response = Json(UserResponse {
        name: params.name,
    });

    debug!(status = ?status, response = ?response);
    (status, response)
}

pub async fn get_user(
        State(_application): State<Arc<Application>>,
        Path(id): Path<Uuid>,
) -> (StatusCode, Json<UserResponse>) {
    let status = StatusCode::OK;
    let response = Json(UserResponse {
        name: format!("User {}", id),
    });

    debug!(status = ?status, response = ?response);
    (status, response)
}
