use axum::{
    Json,
    extract::{FromRequest, Request},
};
use serde::de::DeserializeOwned;

use crate::error::Error;

pub struct ApiJson<T>(pub T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Error;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Json::<T>::from_request(req, state)
            .await
            .map(|Json(v)| ApiJson(v))
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::header};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestDto {
        name: String,
    }

    #[tokio::test]
    async fn should_extract_valid_json_payload() {
        let request = Request::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"name":"Alice"}"#))
            .unwrap();

        let ApiJson(payload) = ApiJson::<TestDto>::from_request(request, &())
            .await
            .unwrap();

        assert_eq!(payload.name, "Alice");
    }

    #[tokio::test]
    async fn should_reject_invalid_json_payload() {
        let request = Request::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"name":"Alice""#))
            .unwrap();

        let result = ApiJson::<TestDto>::from_request(request, &()).await;

        assert!(matches!(result, Err(Error::Json(_))));
    }
}
