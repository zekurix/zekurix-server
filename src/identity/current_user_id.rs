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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    #[tokio::test]
    async fn should_extract_current_user_id_from_request_extensions() {
        let user_id = UserId::new();

        let mut request = Request::builder().body(()).unwrap();

        request.extensions_mut().insert(CurrentUserId(user_id));

        let (mut parts, _) = request.into_parts();

        let current_user = CurrentUserId::from_request_parts(&mut parts, &())
            .await
            .unwrap();

        assert_eq!(current_user.0, user_id);
    }

    #[tokio::test]
    async fn should_reject_when_current_user_id_is_missing() {
        let request = Request::builder().body(()).unwrap();

        let (mut parts, _) = request.into_parts();

        let result = CurrentUserId::from_request_parts(&mut parts, &()).await;

        assert!(matches!(result, Err(Error::AuthenticationRequired)));
    }
}
