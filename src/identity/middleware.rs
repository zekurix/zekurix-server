use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

use crate::Application;
use crate::error::{Error, Result};

use super::CurrentUserId;

fn bearer_token(request: &Request) -> Result<&str> {
    let header = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(Error::AuthenticationRequired)?;

    let header = header
        .to_str()
        .map_err(|_| Error::InvalidAuthorizationHeader)?;

    header
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
        .ok_or(Error::InvalidAuthorizationHeader)
}

pub async fn authenticate(
    State(application): State<Arc<Application>>,
    mut request: Request,
    next: Next,
) -> Result<Response> {
    let token = bearer_token(&request)?;
    let user_id = application
        .services
        .authentication
        .authenticate(token)
        .await?;

    request.extensions_mut().insert(CurrentUserId(user_id));
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::HeaderValue};
    use test_case::test_case;

    #[test]
    fn should_extract_bearer_token() {
        let request = Request::builder()
            .header(header::AUTHORIZATION, "Bearer token")
            .body(Body::empty())
            .unwrap();

        let token = bearer_token(&request).unwrap();

        assert_eq!(token, "token");
    }

    #[test]
    fn should_reject_missing_auth_header() {
        let request = Request::builder().body(Body::empty()).unwrap();

        let result = bearer_token(&request);

        assert!(matches!(result, Err(Error::AuthenticationRequired)));
    }

    #[test_case(""; "missing bearer prefix")]
    #[test_case(" "; "space bearer prefix")]
    #[test_case("Bearer"; "missing space")]
    #[test_case("Bearer: "; "invalid prefix format")]
    #[test_case("Foo "; "invalid prefix")]
    #[test_case("bearer "; "lowercase bearer")]
    #[test_case("BEARER "; "uppercase bearer")]
    fn should_reject_invalid_auth_header(prefix: &str) {
        let header = format!("{prefix}token");
        let request = Request::builder()
            .header(header::AUTHORIZATION, header)
            .body(Body::empty())
            .unwrap();

        let result = bearer_token(&request);

        assert!(matches!(result, Err(Error::InvalidAuthorizationHeader)));
    }

    #[test]
    fn should_reject_empty_bearer_token() {
        let request = Request::builder()
            .header(header::AUTHORIZATION, "Bearer ")
            .body(Body::empty())
            .unwrap();

        let result = bearer_token(&request);

        assert!(matches!(result, Err(Error::InvalidAuthorizationHeader)));
    }

    #[test]
    fn should_reject_auth_header_with_invalid_utf8() {
        let mut request = Request::builder().body(Body::empty()).unwrap();

        request.headers_mut().insert(
            header::AUTHORIZATION,
            HeaderValue::from_bytes(&[0xFF]).unwrap(),
        );

        let result = bearer_token(&request);

        assert!(matches!(result, Err(Error::InvalidAuthorizationHeader)));
    }
}
