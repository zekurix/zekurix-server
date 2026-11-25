use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::de::DeserializeOwned;

use crate::error::Error;

pub struct ApiQuery<T>(pub T);

impl<S, T> FromRequestParts<S> for ApiQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|Query(v)| ApiQuery(v))
            .map_err(|rejection| Error::InvalidQuery(rejection.body_text()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestQuery {
        limit: u32,
    }

    #[tokio::test]
    async fn should_extract_valid_query() {
        let request = Request::builder().uri("/?limit=42").body(()).unwrap();

        let (mut parts, _) = request.into_parts();

        let ApiQuery(query) = ApiQuery::<TestQuery>::from_request_parts(&mut parts, &())
            .await
            .unwrap();

        assert_eq!(query.limit, 42);
    }

    #[tokio::test]
    async fn should_reject_invalid_query() {
        let request = Request::builder().uri("/?limit=invalid").body(()).unwrap();

        let (mut parts, _) = request.into_parts();

        let result = ApiQuery::<TestQuery>::from_request_parts(&mut parts, &()).await;

        assert!(matches!(result, Err(Error::InvalidQuery(_))));
    }
}
