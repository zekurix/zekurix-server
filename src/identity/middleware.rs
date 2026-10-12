use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::Application;
use crate::error::{Error, Result};
use crate::user::{User, UserRepository};

use super::{Identity, IdentityRepository, Issuer, Subject};

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
    /*mut */ request: Request,
    next: Next,
) -> Result<Response> {
    let token = bearer_token(&request)?;
    let claims = application.identity_service.decode(token)?;
    let issuer = Issuer::new(&claims.iss)?;
    let subject = Subject::new(&claims.sub)?;
    match application
        .repositories
        .identity
        .find(&issuer, &subject)
        .await
    {
        Ok(_) => {}
        Err(Error::IdentityNotFound { .. }) => {
            let user = User::new();
            let identity = Identity::new(issuer, subject, user.id);
            application.repositories.user.create(user, identity).await?;
        }
        Err(error) => return Err(error),
    }
    Ok(next.run(request).await)
}
