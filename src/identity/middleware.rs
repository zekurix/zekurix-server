use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::Application;
use crate::error::{Error, Result};

fn bearer_token(request: &Request) -> Result<&str> {
    let header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
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
    State(_application): State<Arc<Application>>,
    /*mut */ request: Request,
    next: Next,
) -> Result<Response> {
    tracing::debug!("Authenticating request: {:?}", request);
    let _token = bearer_token(&request);
    // let token = bearer_token(&request)?;
    Ok(next.run(request).await)
}
