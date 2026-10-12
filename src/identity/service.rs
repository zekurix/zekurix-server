use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use crate::error::{Error, Result};
use crate::settings::authentication::Settings;

use super::claims::Claims;

pub struct Service {
    validation: Validation,
    decoding_key: DecodingKey,
}

impl Service {
    pub fn new(settings: &Settings) -> Self {
        let secret = settings.issuer.repeat(2);

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["iss", "sub", "aud", "exp"]);
        validation.leeway = settings.clock_skew.as_secs();
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.validate_aud = true;
        validation.set_audience(std::slice::from_ref(&settings.audience));
        validation.set_issuer(std::slice::from_ref(&settings.issuer));

        Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            validation,
        }
    }

    pub fn decode(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|token| token.claims)
            .map_err(|_| Error::InvalidToken)
    }
}
