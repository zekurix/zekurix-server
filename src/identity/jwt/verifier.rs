use std::sync::Arc;

use jsonwebtoken::{Validation, decode, decode_header};
use tracing::trace;

use crate::error::{Error, Result};
use crate::identity::VerificationKeyStore;
use crate::settings::authentication::Settings;

use super::Claims;

pub struct JwtVerifier {
    store: Arc<VerificationKeyStore>,
    validation: Validation,
}

impl JwtVerifier {
    pub fn new(settings: &Settings, store: Arc<VerificationKeyStore>) -> Self {
        let algorithms = settings
            .allowed_algorithms
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<jsonwebtoken::Algorithm>>();

        // Settings validation ensures allowed_algorithms is not empty, so algorithms[0] is safe.
        let mut validation = Validation::new(algorithms[0]);
        validation.algorithms = algorithms;
        validation.set_required_spec_claims(&["iss", "sub", "aud", "exp"]);
        validation.leeway = settings.clock_skew.as_secs();
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.validate_aud = true;
        validation.set_audience(std::slice::from_ref(&settings.audience));
        validation.set_issuer(std::slice::from_ref(&settings.issuer));

        Self { store, validation }
    }

    pub async fn verify(&self, token: &str) -> Result<Claims> {
        let header = decode_header(token).map_err(|_| Error::InvalidToken)?;

        let keys = self.store.current().await;
        let candidates = keys.find_candidates(header.kid.as_deref(), header.alg);

        for key in candidates {
            if let Ok(token_data) = decode::<Claims>(token, &key.decoding_key, &self.validation) {
                return Ok(token_data.claims);
            }
        }

        trace!(
            kid = header.kid.as_deref(),
            algorithm = ?header.alg,
            "JWT verification failed"
        );

        Err(Error::InvalidToken)
    }
}
