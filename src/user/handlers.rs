use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use super::{User, UserId, repository::UserRepository};
use crate::application::Application;
use crate::error::Result;

#[derive(Deserialize)]
pub struct UserParams {
    username: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    id: UserId,
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
    Path(id): Path<UserId>,
) -> Result<Json<UserResponse>> {
    let user = application.postgres_user_repository.find(id).await?;

    Ok(Json(user.into()))
}

pub async fn create_user(
    State(application): State<Arc<Application>>,
    Json(params): Json<UserParams>,
) -> Result<(StatusCode, Json<UserResponse>)> {
    let user = User::new(params.username);

    application
        .postgres_user_repository
        .create(user.clone())
        .await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}
