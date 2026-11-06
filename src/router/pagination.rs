use axum::{
    extract::{FromRequestParts, Query},
    http::request::Parts,
};
use serde::Deserialize;

use crate::error::Error;

const DEFAULT_PAGE_LIMIT: u32 = 50;
const MAX_PAGE_LIMIT: u32 = 100;

const fn default_page_limit() -> u32 {
    DEFAULT_PAGE_LIMIT
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page_limit")]
    pub limit: u32,

    #[serde(default)]
    pub offset: u64,
}

impl<S> FromRequestParts<S> for Pagination
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(pagination) = Query::<Pagination>::from_request_parts(parts, state)
            .await
            .map_err(|rejection| Error::InvalidQuery(rejection.body_text()))?;

        if pagination.limit == 0 {
            return Err(Error::InvalidQuery(
                "pagination limit must be greater than zero".to_owned(),
            ));
        }

        if pagination.limit > MAX_PAGE_LIMIT {
            return Err(Error::InvalidQuery(format!(
                "pagination limit must not be greater than {MAX_PAGE_LIMIT}"
            )));
        }

        Ok(pagination)
    }
}

#[cfg(test)]
mod tests {
    use axum::http::Request;

    use super::*;

    async fn extract_pagination(uri: &str) -> Result<Pagination, Error> {
        let request = Request::builder().uri(uri).body(()).unwrap();

        let (mut parts, _) = request.into_parts();

        Pagination::from_request_parts(&mut parts, &()).await
    }

    #[tokio::test]
    async fn missing_query_uses_default_values() {
        let pagination = extract_pagination("/").await.unwrap();

        assert_eq!(pagination.limit, DEFAULT_PAGE_LIMIT);
        assert_eq!(pagination.offset, 0);
    }

    #[tokio::test]
    async fn missing_limit_uses_default_limit() {
        let pagination = extract_pagination("/?offset=100").await.unwrap();

        assert_eq!(pagination.limit, DEFAULT_PAGE_LIMIT);
        assert_eq!(pagination.offset, 100);
    }

    #[tokio::test]
    async fn missing_offset_uses_default_offset() {
        let pagination = extract_pagination("/?limit=25").await.unwrap();

        assert_eq!(pagination.limit, 25);
        assert_eq!(pagination.offset, 0);
    }

    #[tokio::test]
    async fn explicit_values_are_extracted() {
        let pagination = extract_pagination("/?limit=25&offset=100").await.unwrap();

        assert_eq!(pagination.limit, 25);
        assert_eq!(pagination.offset, 100);
    }

    #[tokio::test]
    async fn minimum_valid_limit_is_accepted() {
        let pagination = extract_pagination("/?limit=1").await.unwrap();

        assert_eq!(pagination.limit, 1);
        assert_eq!(pagination.offset, 0);
    }

    #[tokio::test]
    async fn maximum_valid_limit_is_accepted() {
        let pagination = extract_pagination("/?limit=100").await.unwrap();

        assert_eq!(pagination.limit, MAX_PAGE_LIMIT);
        assert_eq!(pagination.offset, 0);
    }

    #[tokio::test]
    async fn zero_limit_is_rejected() {
        let error = extract_pagination("/?limit=0").await.unwrap_err();

        assert!(matches!(error, Error::InvalidQuery(_)));
    }

    #[tokio::test]
    async fn limit_greater_than_maximum_is_rejected() {
        let error = extract_pagination("/?limit=101").await.unwrap_err();

        assert!(matches!(error, Error::InvalidQuery(_)));
    }

    #[tokio::test]
    async fn non_numeric_limit_is_rejected() {
        let error = extract_pagination("/?limit=invalid").await.unwrap_err();

        assert!(matches!(error, Error::InvalidQuery(_)));
    }

    #[tokio::test]
    async fn non_numeric_offset_is_rejected() {
        let error = extract_pagination("/?offset=invalid").await.unwrap_err();

        assert!(matches!(error, Error::InvalidQuery(_)));
    }

    #[tokio::test]
    async fn unknown_query_parameter_is_ignored() {
        let pagination = extract_pagination("/?limit=25&offset=100&foo=bar")
            .await
            .unwrap();

        assert_eq!(pagination.limit, 25);
        assert_eq!(pagination.offset, 100);
    }
}
