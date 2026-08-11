use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use crate::{application::Application, user::User};

#[derive(Deserialize)]
pub struct UserParams {
    username: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    id: Uuid,
    username: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
        }
    }
}

pub async fn get_user(
    State(application): State<Arc<Application>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<UserResponse>), StatusCode> {
    let user = application.users.find(id).ok_or(StatusCode::NOT_FOUND)?;

    let status = StatusCode::OK;
    let response = Json(user.into());

    debug!(status = ?status, response = ?response);
    Ok((status, response))
}

pub async fn create_user(
    State(application): State<Arc<Application>>,
    Json(params): Json<UserParams>,
) -> Result<(StatusCode, Json<UserResponse>), StatusCode> {
    let user = User::new(params.username);

    application
        .users
        .create(user.clone())
        .map_err(|_| StatusCode::CONFLICT)?;

    let status = StatusCode::CREATED;
    let response = Json(user.into());

    debug!(status = ?status, response = ?response);
    Ok((status, response))
}
