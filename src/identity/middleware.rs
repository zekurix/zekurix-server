use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::Application;
use crate::error::{Error, Result};

use super::CurrentUserId;

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
