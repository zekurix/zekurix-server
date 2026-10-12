use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::Application;
use crate::error::{Error, Result};
use crate::user::{User, UserRepository};

use super::{CurrentUserId, Identity, IdentityRepository, Issuer, Subject};

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
    let claims = application.identity_service.decode(token)?;
    let issuer = Issuer::new(&claims.iss)?;
    let subject = Subject::new(&claims.sub)?;
    let user_id = match application
        .repositories
        .identity
        .find(&issuer, &subject)
        .await
    {
        Ok(identity) => identity.user_id,
        Err(Error::IdentityNotFound { .. }) => {
            let user = User::new();
            let identity = Identity::new(issuer, subject, user.id);
            let user = application.repositories.user.create(user, identity).await?;
            user.id
        }
        Err(error) => return Err(error),
    };

    request.extensions_mut().insert(CurrentUserId(user_id));
    Ok(next.run(request).await)
}
