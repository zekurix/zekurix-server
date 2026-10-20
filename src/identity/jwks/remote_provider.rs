use std::time::Duration;

use async_trait::async_trait;
use jsonwebtoken::jwk::JwkSet;
use reqwest::Client;
use url::Url;

use crate::error::{Error, Result};

use super::JwksProvider;

pub struct RemoteJwksProvider {
    url: Url,
    client: Client,
}

impl RemoteJwksProvider {
    pub fn new(url: Url, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("failed to create HTTP client");

        Self { url, client }
    }
}

#[async_trait]
impl JwksProvider for RemoteJwksProvider {
    async fn load(&self) -> Result<JwkSet> {
        let response = self
            .client
            .get(self.url.as_str())
            .send()
            .await
            .map_err(|e| Error::JwksLoadFailed(e.into()))?
            .error_for_status()
            .map_err(|e| Error::JwksLoadFailed(e.into()))?;

        let jwks = response
            .json::<JwkSet>()
            .await
            .map_err(|e| Error::JwksLoadFailed(e.into()))?;

        Ok(jwks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    const VALID_JWKS: &str = include_str!("../../../tests/keys/authentication/jwks.json");

    #[tokio::test]
    async fn should_load_jwks_from_remote_endpoint() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/jwks"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(VALID_JWKS, "application/json"))
            .mount(&server)
            .await;

        let provider = RemoteJwksProvider::new(
            Url::parse(&format!("{}/jwks", server.uri())).unwrap(),
            Duration::from_secs(10),
        );

        let jwks = provider.load().await.unwrap();

        assert!(!jwks.keys.is_empty());
    }

    #[tokio::test]
    async fn should_fail_when_remote_endpoint_returns_error_status() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/jwks"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let provider = RemoteJwksProvider::new(
            Url::parse(&format!("{}/jwks", server.uri())).unwrap(),
            Duration::from_secs(10),
        );

        let result = provider.load().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
    }

    #[tokio::test]
    async fn should_fail_when_remote_endpoint_returns_invalid_json() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/jwks"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw("invalid JSON", "application/json"),
            )
            .mount(&server)
            .await;

        let provider = RemoteJwksProvider::new(
            Url::parse(&format!("{}/jwks", server.uri())).unwrap(),
            Duration::from_secs(10),
        );

        let result = provider.load().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
    }

    #[tokio::test]
    async fn should_fail_when_remote_endpoint_times_out() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/jwks"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(100))
                    .set_body_raw(VALID_JWKS, "application/json"),
            )
            .mount(&server)
            .await;

        let provider = RemoteJwksProvider::new(
            Url::parse(&format!("{}/jwks", server.uri())).unwrap(),
            Duration::from_millis(10),
        );

        let result = provider.load().await;

        assert!(matches!(result, Err(Error::JwksLoadFailed(_))));
    }
}
