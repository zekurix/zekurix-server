use axum::{extract::FromRequestParts, http::request::Parts};

use crate::error::Error;
use crate::user::UserId;

#[derive(Clone, Copy, Debug)]
pub struct CurrentUserId(pub UserId);

impl<S> FromRequestParts<S> for CurrentUserId
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUserId>()
            .copied()
            .ok_or(Error::AuthenticationRequired)
    }
}
