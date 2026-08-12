use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::Application;
use crate::error::Result;
use crate::user::User;

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
) -> Result<Json<UserResponse>> {
    let user = application.users.find(id)?;

    Ok(Json(user.into()))
}

pub async fn create_user(
    State(application): State<Arc<Application>>,
    Json(params): Json<UserParams>,
) -> Result<(StatusCode, Json<UserResponse>)> {
    let user = User::new(params.username);

    application.users.create(user.clone())?;

    Ok((StatusCode::CREATED, Json(user.into())))
}
