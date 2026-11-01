use std::sync::Arc;

use async_trait::async_trait;
use jsonwebtoken::jwk::JwkSet;

use crate::error::Result;
use crate::settings::authentication_provider::Settings;

use super::{FileJwksProvider, RemoteJwksProvider};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait JwksProvider: Send + Sync {
    async fn load(&self) -> Result<JwkSet>;
}

pub fn create_jwks_provider(settings: &Settings) -> Arc<dyn JwksProvider> {
    match (settings.jwks_url.as_ref(), settings.jwks_file.as_ref()) {
        (Some(url), None) => Arc::new(RemoteJwksProvider::new(
            url.clone(),
            settings.request_timeout,
        )),
        (None, Some(path)) => Arc::new(FileJwksProvider::new(path.to_path_buf())),
        _ => panic!("exactly one of jwks_url or jwks_file must be configured"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_remote_jwks_provider_when_jwks_url_is_configured() {
        let settings = Settings {
            jwks_url: Some("https://auth.example.com/my-issuer/jwks".parse().unwrap()),
            jwks_file: None,
            ..Default::default()
        };

        create_jwks_provider(&settings);
    }

    #[test]
    fn should_create_file_jwks_provider_when_jwks_file_is_configured() {
        let settings = Settings {
            jwks_url: None,
            jwks_file: Some("tests/keys/authentication/jwks.json".into()),
            ..Default::default()
        };

        create_jwks_provider(&settings);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_no_provider_is_configured() {
        let settings = Settings {
            jwks_url: None,
            jwks_file: None,
            ..Default::default()
        };

        create_jwks_provider(&settings);
    }

    #[test]
    #[should_panic]
    fn should_panic_when_both_providers_are_configured() {
        let settings = Settings {
            jwks_url: Some("https://auth.example.com/my-issuer/jwks".parse().unwrap()),
            jwks_file: Some("tests/keys/authentication/jwks.json".into()),
            ..Default::default()
        };

        create_jwks_provider(&settings);
    }
}
