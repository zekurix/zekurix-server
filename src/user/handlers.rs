use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderName, StatusCode, header},
    response::AppendHeaders,
};

use crate::Application;
use crate::error::Result;
use crate::router::{ApiJson, ApiPath};

use super::{
    User, UserId,
    dto::{CreateUserRequest, UserResponse},
    repository::UserRepository,
};

pub async fn get_user(
    State(application): State<Arc<Application>>,
    ApiPath(id): ApiPath<UserId>,
) -> Result<Json<UserResponse>> {
    let user = application.repositories.user.find(id).await?;

    Ok(Json(user.into()))
}

pub async fn create_user(
    State(application): State<Arc<Application>>,
    ApiJson(params): ApiJson<CreateUserRequest>,
) -> Result<(
    StatusCode,
    AppendHeaders<[(HeaderName, String); 1]>,
    Json<UserResponse>,
)> {
    let user = User::new(params.username);
    let user = application.repositories.user.create(user).await?;

    Ok((
        StatusCode::CREATED,
        AppendHeaders([(header::LOCATION, format!("/api/v1/users/{}", user.id))]),
        Json(user.into()),
    ))
}
