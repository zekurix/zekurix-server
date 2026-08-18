use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("internal error")]
    InternalError,

    #[error("user '{0}' already exists")]
    UserAlreadyExists(String),

    #[error("user '{0}' not found")]
    UserNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: &'static str,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Error::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            Error::UserAlreadyExists(_) => (StatusCode::CONFLICT, "USER_ALREADY_EXISTS"),
            Error::UserNotFound(_) => (StatusCode::NOT_FOUND, "USER_NOT_FOUND"),
        };

        let body = Json(ErrorResponse { code });

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_map_internal_error_to_500() {
        let response = Error::InternalError.into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn should_map_user_already_exists_to_409() {
        let response = Error::UserAlreadyExists("alice".into()).into_response();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn should_map_user_not_found_to_404() {
        let response = Error::UserNotFound("alice".into()).into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
