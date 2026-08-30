use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::Application;
use crate::error::Result;

use super::{User, UserId, Username, repository::UserRepository};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    username: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: UserId,
    username: Username,
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
    let user = application.repositories.user.find(id).await?;

    Ok(Json(user.into()))
}

pub async fn create_user(
    State(application): State<Arc<Application>>,
    Json(params): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>)> {
    let username = Username::new(&params.username)?;
    let user = User::new(username);

    application.repositories.user.create(user.clone()).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}
