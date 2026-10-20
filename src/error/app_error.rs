use axum::{
    extract::rejection::{JsonRejection, PathRejection},
    http::{
        Method, StatusCode, Uri,
        header::{self, HeaderValue},
    },
    response::{IntoResponse, Response},
};
use problem_details::ProblemDetails;
use thiserror::Error;

use super::problem_type;
use crate::identity::{Identity, Issuer, Subject};
use crate::user::UserId;

#[derive(Debug, Error)]
pub enum Error {
    #[error("internal error")]
    InternalError(#[source] anyhow::Error),

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

    #[error("identity '{0}' already exists")]
    IdentityAlreadyExists(Identity),

    #[error("identity '{subject}@{issuer}' not found")]
    IdentityNotFound { issuer: Issuer, subject: Subject },

    #[error("user '{0}' not found")]
    UserNotFound(UserId),

    #[error("issuer '{0}' invalid")]
    InvalidIssuer(String),

    #[error("subject '{0}' invalid")]
    InvalidSubject(String),

    #[error("JSON error")]
    Json(#[from] JsonRejection),

    #[error("Path error")]
    Path(#[from] PathRejection),

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Invalid authorization header")]
    InvalidAuthorizationHeader,

    #[error("Invalid token")]
    InvalidToken,

    #[error("failed to load JWKS")]
    JwksLoadFailed(#[source] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

fn internal_server_error() -> ProblemDetails {
    ProblemDetails::new()
        .with_type(problem_type::INTERNAL_SERVER_ERROR.as_uri())
        .with_title("Internal Server Error")
        .with_status(StatusCode::INTERNAL_SERVER_ERROR)
        .with_detail("An unexpected internal server error occurred.")
}

impl From<Error> for ProblemDetails {
    fn from(error: Error) -> Self {
        match error {
            Error::HttpNotFound(uri) => ProblemDetails::new()
                .with_type(problem_type::http::NOT_FOUND.as_uri())
                .with_title("Resource Not Found")
                .with_status(StatusCode::NOT_FOUND)
                .with_detail(format!("Resource '{uri}' was not found.")),

            Error::HttpMethodNotAllowed { uri, method } => ProblemDetails::new()
                .with_type(problem_type::http::METHOD_NOT_ALLOWED.as_uri())
                .with_title("Method Not Allowed")
                .with_status(StatusCode::METHOD_NOT_ALLOWED)
                .with_detail(format!(
                    "HTTP method '{method}' not allowed for URI '{uri}'"
                )),

            Error::HttpGatewayTimeout => ProblemDetails::new()
                .with_type(problem_type::http::GATEWAY_TIMEOUT.as_uri())
                .with_title("Gateway Timeout")
                .with_status(StatusCode::GATEWAY_TIMEOUT)
                .with_detail("Gateway timed out."),

            Error::IdentityAlreadyExists(identity) => ProblemDetails::new()
                .with_type(problem_type::user::ALREADY_EXISTS.as_uri())
                .with_title("User Already Exists")
                .with_status(StatusCode::CONFLICT)
                .with_detail(format!(
                    "An account associated with identity '{identity}' already exists."
                )),

            Error::UserNotFound(id) => ProblemDetails::new()
                .with_type(problem_type::user::NOT_FOUND.as_uri())
                .with_title("User Not Found")
                .with_status(StatusCode::NOT_FOUND)
                .with_detail(format!("User '{id}' was not found.")),

            Error::InvalidIssuer(_) => ProblemDetails::new()
                .with_type(problem_type::http::UNAUTHORIZED.as_uri())
                .with_title("Invalid Identity Issuer")
                .with_status(StatusCode::UNAUTHORIZED)
                .with_detail("The identity issuer is invalid."),

            Error::InvalidSubject(_) => ProblemDetails::new()
                .with_type(problem_type::http::UNAUTHORIZED.as_uri())
                .with_title("Invalid Identity Subject")
                .with_status(StatusCode::UNAUTHORIZED)
                .with_detail("The identity subject is invalid."),

            Error::Json(rejection) => match rejection {
                JsonRejection::JsonDataError(_) => ProblemDetails::new()
                    .with_type(problem_type::json::DATA_ERROR.as_uri())
                    .with_title("JSON rejection: data error")
                    .with_status(rejection.status())
                    .with_detail(rejection.body_text()),

                JsonRejection::JsonSyntaxError(_) => ProblemDetails::new()
                    .with_type(problem_type::json::SYNTAX_ERROR.as_uri())
                    .with_title("JSON rejection: syntax error")
                    .with_status(rejection.status())
                    .with_detail(rejection.body_text()),

                JsonRejection::MissingJsonContentType(_) => ProblemDetails::new()
                    .with_type(problem_type::json::MISSING_CONTENT_TYPE.as_uri())
                    .with_title("JSON rejection: missing content type")
                    .with_status(rejection.status())
                    .with_detail(rejection.body_text()),

                JsonRejection::BytesRejection(_) => ProblemDetails::new()
                    .with_type(problem_type::json::BYTES_REJECTION.as_uri())
                    .with_title("JSON rejection: bytes rejection")
                    .with_status(rejection.status())
                    .with_detail(rejection.body_text()),

                _ => internal_server_error(),
            },

            Error::Path(rejection) => ProblemDetails::new()
                .with_type(problem_type::path::REJECTION.as_uri())
                .with_title("Path rejection")
                .with_status(rejection.status())
                .with_detail(rejection.body_text()),

            Error::AuthenticationRequired => ProblemDetails::new()
                .with_type(problem_type::http::UNAUTHORIZED.as_uri())
                .with_title("Authentication required")
                .with_status(StatusCode::UNAUTHORIZED)
                .with_detail("Authentication is required to access this resource."),

            Error::InvalidAuthorizationHeader => ProblemDetails::new()
                .with_type(problem_type::http::UNAUTHORIZED.as_uri())
                .with_title("Invalid Authorization Header")
                .with_status(StatusCode::UNAUTHORIZED)
                .with_detail("The Authorization header is invalid."),

            Error::InvalidToken => ProblemDetails::new()
                .with_type(problem_type::http::UNAUTHORIZED.as_uri())
                .with_title("Invalid Token")
                .with_status(StatusCode::UNAUTHORIZED)
                .with_detail("The access token is invalid."),

            _ => internal_server_error(),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let problem = ProblemDetails::from(self);
        let is_unauthorized = problem.status == Some(StatusCode::UNAUTHORIZED);
        let mut response = problem.into_response();

        if is_unauthorized {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{Issuer, Subject};
    use axum::{
        body::Body,
        extract::{FromRequest, Json, rejection::MissingPathParams},
        http::Request,
    };
    use problem_details;
    use test_case::test_case;

    #[test]
    fn http_not_found_maps_to_not_found() {
        let uri = "/invalid/route".parse::<Uri>().unwrap();
        let error = Error::HttpNotFound(uri);
        let problem = ProblemDetails::from(error);

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
        let error = Error::HttpMethodNotAllowed {
            uri: "/health".parse::<Uri>().unwrap(),
            method: "POST".parse::<Method>().unwrap(),
        };
        let problem = ProblemDetails::from(error);

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
        let error = Error::HttpGatewayTimeout;
        let problem = ProblemDetails::from(error);

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
        let error = Error::from(rejection);
        let problem = ProblemDetails::from(error);

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
        let error = Error::from(rejection);
        let problem = ProblemDetails::from(error);

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
        let rejection = JsonRejection::MissingJsonContentType(Default::default());
        let error = Error::from(rejection);
        let problem = ProblemDetails::from(error);

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
        let error = Error::from(rejection);
        let problem = ProblemDetails::from(error);

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
    fn path_rejection_maps_to_bad_request() {
        let rejection = PathRejection::MissingPathParams(MissingPathParams::default());
        let error = Error::Path(rejection);
        let problem = ProblemDetails::from(error);

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::path::REJECTION.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Path rejection".to_string()));
        assert_eq!(problem.status, Some(StatusCode::INTERNAL_SERVER_ERROR));
        assert!(problem.detail.is_some_and(|s| !s.is_empty()));
    }

    #[test]
    fn identity_already_exists_maps_to_conflict() {
        let identity = Identity::new(
            Issuer::new("https://auth.example.com").unwrap(),
            Subject::new("Alice").unwrap(),
            UserId::new(),
        );
        let error = Error::IdentityAlreadyExists(identity);
        let problem = ProblemDetails::from(error);

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
            Some("An account associated with identity 'Alice@https://auth.example.com' already exists.".to_string())
        );
    }

    #[test]
    fn authentication_required_maps_to_unauthorized() {
        let error = Error::AuthenticationRequired;
        let problem = ProblemDetails::from(error);

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::http::UNAUTHORIZED.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Authentication required".to_string()));
        assert_eq!(problem.status, Some(StatusCode::UNAUTHORIZED));
        assert_eq!(
            problem.detail,
            Some("Authentication is required to access this resource.".to_string())
        );
    }

    #[test]
    fn invalid_authorization_header_maps_to_unauthorized() {
        let error = Error::InvalidAuthorizationHeader;
        let problem = ProblemDetails::from(error);

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::http::UNAUTHORIZED.as_uri()
            ))
        );
        assert_eq!(
            problem.title,
            Some("Invalid Authorization Header".to_string())
        );
        assert_eq!(problem.status, Some(StatusCode::UNAUTHORIZED));
        assert_eq!(
            problem.detail,
            Some("The Authorization header is invalid.".to_string())
        );
    }

    #[test]
    fn invalid_token_maps_to_unauthorized() {
        let error = Error::InvalidToken;
        let problem = ProblemDetails::from(error);

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::http::UNAUTHORIZED.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Invalid Token".to_string()));
        assert_eq!(problem.status, Some(StatusCode::UNAUTHORIZED));
        assert_eq!(
            problem.detail,
            Some("The access token is invalid.".to_string())
        );
    }

    #[test]
    fn invalid_issuer_maps_to_unauthorized() {
        let error = Error::InvalidIssuer("Invalid Issuer".to_string());
        let problem = ProblemDetails::from(error);

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::http::UNAUTHORIZED.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Invalid Identity Issuer".to_string()));
        assert_eq!(problem.status, Some(StatusCode::UNAUTHORIZED));
        assert_eq!(
            problem.detail,
            Some("The identity issuer is invalid.".to_string())
        );
    }

    #[test]
    fn invalid_subject_maps_to_unauthorized() {
        let error = Error::InvalidSubject("Invalid Subject".to_string());
        let problem = ProblemDetails::from(error);

        assert_eq!(
            problem.r#type,
            Some(problem_details::ProblemType::from(
                problem_type::http::UNAUTHORIZED.as_uri()
            ))
        );
        assert_eq!(problem.title, Some("Invalid Identity Subject".to_string()));
        assert_eq!(problem.status, Some(StatusCode::UNAUTHORIZED));
        assert_eq!(
            problem.detail,
            Some("The identity subject is invalid.".to_string())
        );
    }

    #[test]
    fn user_not_found_maps_to_not_found() {
        let id = UserId::new();
        let error = Error::UserNotFound(id);
        let problem = ProblemDetails::from(error);

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

    #[test_case(Error::InternalError(anyhow::anyhow!("error")); "internal server error")]
    #[test_case(Error::IdentityNotFound {
            issuer: Issuer::new("issuer").unwrap(),
            subject: Subject::new("subject").unwrap(),
        }; "identity not found")]
    #[test_case(Error::MissingEnvironmentVariable("ZEKURIX_DATABASE__PASSWORD".to_owned()); "missing environment variable")]
    #[test_case(Error::InvalidEnvironmentVariable("ZEKURIX_DATABASE__PASSWORD".to_owned()); "invalid environment variable")]
    #[test_case(Error::InvalidSettings {
            setting: "setting".to_owned(),
            reason: "reason".to_owned(),
        }; "invalid settings")]
    #[test_case(Error::JwksLoadFailed(anyhow::anyhow!("error")); "jwks load failed")]
    fn other_error_maps_to_internal_server_error(error: Error) {
        let problem = ProblemDetails::from(error);

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

    #[test_case(Error::AuthenticationRequired ; "authentication required")]
    #[test_case(Error::InvalidAuthorizationHeader ; "invalid authorization header")]
    #[test_case(Error::InvalidToken ; "invalid token")]
    fn test_unauthorized_error_includes_www_authenticate_header(error: Error) {
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(response.headers().contains_key(header::WWW_AUTHENTICATE));
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            "Bearer"
        );
    }

    #[test]
    fn test_non_unauthorized_error_no_www_authenticate_header() {
        let error = Error::InternalError(anyhow::anyhow!("error"));
        let response = error.into_response();

        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(!response.headers().contains_key(header::WWW_AUTHENTICATE));
    }
}
