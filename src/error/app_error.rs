use axum::{
    extract::rejection::JsonRejection,
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use problem_details::ProblemDetails;
use thiserror::Error;

use super::problem_type;
use crate::user::{UserId, Username};

#[derive(Debug, Error)]
pub enum Error {
    #[error("internal error")]
    InternalError,

    #[error("environement variable '{0}' missing")]
    MissingEnvironmentVariable(String),

    #[error("environement variable '{0}' invalid")]
    InvalidEnvironmentVariable(String),

    #[error("Resource '{0}' not found.")]
    HttpNotFound(Uri),

    #[error("HTTP method '{method}' not allowed for URI '{uri}'")]
    HttpMethodNotAllowed { uri: Uri, method: Method },

    #[error("Gateway timeout")]
    HttpGatewayTimeout,

    #[error("setting '{setting}' invalid: {reason}")]
    InvalidSettings { setting: String, reason: String },

    #[error("user '{0}' already exists")]
    UserAlreadyExists(Username),

    #[error("user '{0}' not found")]
    UserNotFound(UserId),

    #[error("username '{0}' invalid")]
    InvalidUsername(String),

    #[error("JSON error")]
    Json(#[from] JsonRejection),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    fn into_problem_details(self) -> ProblemDetails {
        match self {
            Self::HttpNotFound(uri) => ProblemDetails::new()
                .with_type(problem_type::http::NOT_FOUND.as_uri())
                .with_title("Resource Not Found")
                .with_status(StatusCode::NOT_FOUND)
                .with_detail(format!("Resource '{uri}' was not found.")),

            Self::HttpMethodNotAllowed { uri, method } => ProblemDetails::new()
                .with_type(problem_type::http::METHOD_NOT_ALLOWED.as_uri())
                .with_title("Method Not Allowed")
                .with_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_detail(format!(
                    "HTTP method '{method}' not allowed for URI '{uri}'"
                )),

            Self::HttpGatewayTimeout => ProblemDetails::new()
                .with_type(problem_type::http::GATEWAY_TIMEOUT.as_uri())
                .with_title("Gateway Timeout")
                .with_status(StatusCode::GATEWAY_TIMEOUT)
                .with_detail("Gateway timed out."),

            Self::UserAlreadyExists(username) => ProblemDetails::new()
                .with_type(problem_type::user::ALREADY_EXISTS.as_uri())
                .with_title("User Already Exists")
                .with_status(StatusCode::CONFLICT)
                .with_detail(format!("User '{username}' already exists.")),

            Self::UserNotFound(id) => ProblemDetails::new()
                .with_type(problem_type::user::NOT_FOUND.as_uri())
                .with_title("User Not Found")
                .with_status(StatusCode::NOT_FOUND)
                .with_detail(format!("User '{id}' was not found.")),

            Self::InvalidUsername(username) => ProblemDetails::new()
                .with_type(problem_type::user::INVALID_USERNAME.as_uri())
                .with_title("Invalid Username")
                .with_status(StatusCode::UNPROCESSABLE_ENTITY)
                .with_detail(format!("Username '{username}' is invalid.")),

            Self::Json(json_rejection) => match json_rejection {
                JsonRejection::JsonDataError(_) => ProblemDetails::new()
                    .with_type(problem_type::json::DATA_ERROR.as_uri())
                    .with_title("JSON rejection: data error")
                    .with_status(json_rejection.status())
                    .with_detail(json_rejection.body_text()),

                JsonRejection::JsonSyntaxError(_) => ProblemDetails::new()
                    .with_type(problem_type::json::SYNTAX_ERROR.as_uri())
                    .with_title("JSON rejection: syntax error")
                    .with_status(json_rejection.status())
                    .with_detail(json_rejection.body_text()),

                JsonRejection::MissingJsonContentType(_) => ProblemDetails::new()
                    .with_type(problem_type::json::MISSING_CONTENT_TYPE.as_uri())
                    .with_title("JSON rejection: missing content type")
                    .with_status(json_rejection.status())
                    .with_detail(json_rejection.body_text()),

                JsonRejection::BytesRejection(_) => ProblemDetails::new()
                    .with_type(problem_type::json::BYTES_REJECTION.as_uri())
                    .with_title("JSON rejection: bytes rejection")
                    .with_status(json_rejection.status())
                    .with_detail(json_rejection.body_text()),

                _ => ProblemDetails::new()
                    .with_type(problem_type::INTERNAL_SERVER_ERROR.as_uri())
                    .with_title("Internal Server Error")
                    .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                    .with_detail("An unexpected internal server error occurred."),
            },

            _ => ProblemDetails::new()
                .with_type(problem_type::INTERNAL_SERVER_ERROR.as_uri())
                .with_title("Internal Server Error")
                .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                .with_detail("An unexpected internal server error occurred."),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        self.into_problem_details().into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::{FromRequest, Json},
        http::Request,
    };
    use problem_details;
    use test_case::test_case;

    #[test]
    fn http_not_found_maps_to_not_found() {
        let uri = "/invalid/route".parse::<Uri>().unwrap();

        let problem = Error::HttpNotFound(uri).into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::http::NOT_FOUND.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Resource Not Found".to_string()));
        assert_eq!(problem.status, Some(StatusCode::NOT_FOUND));
        assert_eq!(
            problem.detail,
            Some("Resource '/invalid/route' was not found.".to_string())
        );
    }

    #[test]
    fn http_method_not_allowed_maps_to_method_not_allowed() {
        let uri = "/health".parse::<Uri>().unwrap();
        let method = "POST".parse::<Method>().unwrap();

        let problem = Error::HttpMethodNotAllowed { uri, method }.into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::http::METHOD_NOT_ALLOWED.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Method Not Allowed".to_string()));
        assert_eq!(problem.status, Some(StatusCode::METHOD_NOT_ALLOWED));
        assert_eq!(
            problem.detail,
            Some("HTTP method 'POST' not allowed for URI '/health'".to_string())
        );
    }

    #[test]
    fn http_gateway_timeout_maps_to_gateway_timeout() {
        let problem = Error::HttpGatewayTimeout.into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::http::GATEWAY_TIMEOUT.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Gateway Timeout".to_string()));
        assert_eq!(problem.status, Some(StatusCode::GATEWAY_TIMEOUT));
        assert_eq!(problem.detail, Some("Gateway timed out.".to_string()));
    }

    #[tokio::test]
    async fn json_data_error_maps_to_unprocessable_entity() {
        let rejection = Json::<String>::from_request(
            Request::builder()
                .header("content-type", "application/json")
                .body(Body::from("42"))
                .unwrap(),
            &(),
        )
        .await
        .unwrap_err();
        let err = Error::from(rejection);
        let problem = err.into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::json::DATA_ERROR.as_uri()
            ))
        );
        assert_eq!(
            problem.title,
            Some("JSON rejection: data error".to_string())
        );
        assert_eq!(problem.status, Some(StatusCode::UNPROCESSABLE_ENTITY));
        assert!(problem.detail.is_some_and(|s| !s.is_empty()));
    }

    #[tokio::test]
    async fn json_syntax_error_maps_to_bad_request() {
        let rejection = Json::<String>::from_request(
            Request::builder()
                .header("content-type", "application/json")
                .body(Body::from("not a json"))
                .unwrap(),
            &(),
        )
        .await
        .unwrap_err();
        let app_err = Error::from(rejection);
        let problem = app_err.into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::json::SYNTAX_ERROR.as_uri()
            ))
        );
        assert_eq!(
            problem.title,
            Some("JSON rejection: syntax error".to_string())
        );
        assert_eq!(problem.status, Some(StatusCode::BAD_REQUEST));
        assert!(problem.detail.is_some_and(|s| !s.is_empty()));
    }

    #[test]
    fn json_missing_content_type_maps_to_unsupported_media_type() {
        let jr = JsonRejection::MissingJsonContentType(Default::default());
        let app_err = Error::from(jr);
        let problem = app_err.into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::json::MISSING_CONTENT_TYPE.as_uri()
            ))
        );
        assert_eq!(
            problem.title,
            Some("JSON rejection: missing content type".to_string())
        );
        assert_eq!(problem.status, Some(StatusCode::UNSUPPORTED_MEDIA_TYPE));
        assert!(problem.detail.is_some_and(|s| !s.is_empty()));
    }

    #[tokio::test]
    async fn json_bytes_rejection_maps_to_bad_request() {
        let rejection = Json::<String>::from_request(
            Request::builder()
                .header("content-type", "application/json")
                .body(Body::from_stream(tokio_stream::once(Err::<
                    axum::body::Bytes,
                    _,
                >(
                    std::io::Error::other("body read failed"),
                ))))
                .unwrap(),
            &(),
        )
        .await
        .unwrap_err();
        let app_err = Error::from(rejection);
        let problem = app_err.into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::json::BYTES_REJECTION.as_uri()
            ))
        );
        assert_eq!(
            problem.title,
            Some("JSON rejection: bytes rejection".to_string())
        );
        assert_eq!(problem.status, Some(StatusCode::BAD_REQUEST));
        assert!(problem.detail.is_some_and(|s| !s.is_empty()));
    }

    #[test]
    fn user_already_exists_maps_to_conflict() {
        let username = Username::new("Alice").unwrap();

        let problem = Error::UserAlreadyExists(username).into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::user::ALREADY_EXISTS.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("User Already Exists".to_string()));
        assert_eq!(problem.status, Some(StatusCode::CONFLICT));
        assert_eq!(
            problem.detail,
            Some("User 'Alice' already exists.".to_string())
        );
    }

    #[test]
    fn user_not_found_maps_to_not_found() {
        let id = UserId::new();

        let problem = Error::UserNotFound(id).into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::user::NOT_FOUND.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("User Not Found".to_string()));
        assert_eq!(problem.status, Some(StatusCode::NOT_FOUND));
        assert_eq!(problem.detail, Some(format!("User '{id}' was not found.")));
    }

    #[test]
    fn invalid_username_maps_to_conflict() {
        let username = "invalid username".to_owned();

        let problem = Error::InvalidUsername(username).into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::user::INVALID_USERNAME.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Invalid Username".to_string()));
        assert_eq!(problem.status, Some(StatusCode::UNPROCESSABLE_ENTITY));
        assert_eq!(
            problem.detail,
            Some("Username 'invalid username' is invalid.".to_string())
        );
    }

    #[test_case(Error::InternalError ; "internal server error")]
    #[test_case(Error::MissingEnvironmentVariable("ZEKURIX_DATABASE__PASSWORD".to_owned()) ; "missing environment variable")]
    #[test_case(Error::InvalidEnvironmentVariable("ZEKURIX_DATABASE__PASSWORD".to_owned()) ; "invalid environment variable")]
    #[test_case(Error::InvalidSettings {
            setting: "setting".to_owned(),
            reason: "reason".to_owned(),
        } ; "invalid settings")]
    fn other_error_maps_to_internal_server_error(error: Error) {
        let problem = error.into_problem_details();

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::INTERNAL_SERVER_ERROR.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Internal Server Error".to_string()));
        assert_eq!(problem.status, Some(StatusCode::INTERNAL_SERVER_ERROR));
        assert_eq!(
            problem.detail,
            Some("An unexpected internal server error occurred.".to_string())
        );
    }
}
