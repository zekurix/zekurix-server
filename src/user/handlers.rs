use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::Application;
use crate::error::{ErrorResponse, Result};

use super::{User, UserId, Username, repository::UserRepository};

/// Request parameters for creating a new user.
#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    /// The username for the new user.
    #[schema(example = "Alice")]
    username: String,
}

/// Response containing user information.
#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    /// The unique identifier of the user.
    #[schema(example = "01a04f48-841a-7f60-a628-089369d1da93")]
    id: UserId,

    /// The username of the user.
    #[schema(example = "Alice")]
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

/// Retrieve a user by ID.
///
/// Returns the user information for the specified user ID.
#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "users",
    params(
        (
            "id" = UserId,
            Path,
            description = "The unique identifier of the user"
        )
    ),
    responses(
        (
            status = 200,
            description = "User retrieved successfully",
            body = UserResponse
        ),
        (
            status = 404,
            description = "User not found",
            body = ErrorResponse
        ),
        (
            status = 500,
            description = "An unexpected internal server error occurred",
            body = ErrorResponse
        ),
    )
)]
pub async fn get_user(
    State(application): State<Arc<Application>>,
    Path(id): Path<UserId>,
) -> Result<Json<UserResponse>> {
    let user = application.repositories.user.find(id).await?;

    Ok(Json(user.into()))
}

/// Create a new user.
///
/// Creates a new user with the provided username and returns the created user information.
#[utoipa::path(
    post,
    path = "/users",
    tag = "users",
    request_body(
        content = CreateUserRequest,
        description = "The user parameters required to create the user"
    ),
    responses(
        (
            status = 201,
            description = "User created successfully",
            body = UserResponse
        ),
        (
            status = 409,
            description = "A user with this username already exists",
            body = ErrorResponse
        ),
        (
            status = 422,
            description = "The username is invalid. It must be between 3 and 64 characters and may only contain ASCII alphanumeric characters, '_' and '-'",
            body = ErrorResponse
        ),
        (
            status = 500,
            description = "An unexpected internal server error occurred",
            body = ErrorResponse
        ),
    )
)]
pub async fn create_user(
    State(application): State<Arc<Application>>,
    Json(params): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>)> {
    let username = Username::new(&params.username)?;
    let user = User::new(username);

    application.repositories.user.create(user.clone()).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}
