use std::sync::Arc;

use jsonwebtoken::jwk::JwkSet;

use crate::settings::authentication_provider::Settings;

use super::{JwksProvider, create_jwks_provider};

pub struct JwksService {
    provider: Arc<dyn JwksProvider>,
    jwks: JwkSet,
}

impl JwksService {
    pub fn new(settings: &Settings) -> Self {
        let provider = create_jwks_provider(settings);
        let jwks = JwkSet::default();

        Self { provider, jwks }
    }

    pub async fn dummy(&mut self) {
        self.jwks = self.provider.load().await.unwrap(); // fake to use the code
        tracing::trace!("{:?}", self.jwks);
    }
}
