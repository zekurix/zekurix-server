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
    name: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    id: Uuid,
    name: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            name: user.name,
        }
    }
}

pub async fn create_user(
    State(application): State<Arc<Application>>,
    Json(params): Json<UserParams>,
) -> (StatusCode, Json<UserResponse>) {
    let user = User::new(params.name);

    application.users.create(user.clone());

    let status = StatusCode::CREATED;
    let response = Json(user.into());

    debug!(status = ?status, response = ?response);
    (status, response)
}

pub async fn get_user(
    State(application): State<Arc<Application>>,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<UserResponse>) {
    match application.users.find(id) {
        Some(user) => {
            let status = StatusCode::OK;
            let response = Json(user.into());
            debug!(status = ?status, response = ?response);
            (status, response)
        }
        None => {
            let status = StatusCode::NOT_FOUND;
            let fake = UserResponse {
                id: Uuid::new_v4(),
                name: "".into(),
            };
            let response = Json(fake);
            debug!(status = ?status, response = ?response);
            (status, response)
        }
    }
}
