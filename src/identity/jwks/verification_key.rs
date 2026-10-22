use jsonwebtoken::{
    Algorithm, DecodingKey,
    jwk::{Jwk, KeyAlgorithm},
};
use tracing::{debug, error, warn};

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub struct VerificationKey {
    pub kid: Option<String>,
    pub algorithm: Option<Algorithm>,
    pub decoding_key: DecodingKey,
}

impl VerificationKey {
    pub fn new(jwk: Jwk) -> Result<Self> {
        let kid = jwk.common.key_id.clone();

        let algorithm = match jwk.common.key_algorithm {
            Some(key_algorithm) => match key_algorithm {
                KeyAlgorithm::RS256 => Some(Algorithm::RS256),
                KeyAlgorithm::RS384 => Some(Algorithm::RS384),
                KeyAlgorithm::RS512 => Some(Algorithm::RS512),
                KeyAlgorithm::PS256 => Some(Algorithm::PS256),
                KeyAlgorithm::PS384 => Some(Algorithm::PS384),
                KeyAlgorithm::PS512 => Some(Algorithm::PS512),
                KeyAlgorithm::ES256 => Some(Algorithm::ES256),
                KeyAlgorithm::ES384 => Some(Algorithm::ES384),
                KeyAlgorithm::EdDSA => Some(Algorithm::EdDSA),
                _ => {
                    warn!(
                        ?kid,
                        ?key_algorithm,
                        "ignoring JWK with unsupported algorithm"
                    );
                    return Err(Error::InvalidJwk(anyhow::anyhow!(
                        "unsupported JWK key algorithm"
                    )));
                }
            },

            None => {
                debug!(?kid, "JWK does not specify an algorithm");
                None
            }
        };

        let decoding_key = DecodingKey::from_jwk(&jwk).map_err(|error| {
            error!(
               ?kid,
               ?algorithm,
               %error,
               "failed to create decoding key from JWK"
            );

            Error::InvalidJwk(error.into())
        })?;

        debug!(?kid, ?algorithm, "verification key created");

        Ok(Self {
            kid,
            algorithm,
            decoding_key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn jwk(algorithm: Option<&str>, kid: Option<&str>) -> Jwk {
        let mut value = json!({
            "kty": "RSA",
            "n": "AQ",
            "e": "AQ",
        });

        if let Some(algorithm) = algorithm {
            value["alg"] = json!(algorithm);
        }

        if let Some(kid) = kid {
            value["kid"] = json!(kid);
        }

        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn should_map_supported_key_algorithms() {
        let supported_algorithms = [
            ("RS256", Algorithm::RS256),
            ("RS384", Algorithm::RS384),
            ("RS512", Algorithm::RS512),
            ("PS256", Algorithm::PS256),
            ("PS384", Algorithm::PS384),
            ("PS512", Algorithm::PS512),
            ("ES256", Algorithm::ES256),
            ("ES384", Algorithm::ES384),
            ("EdDSA", Algorithm::EdDSA),
        ];

        for (key_algorithm, expected_algorithm) in supported_algorithms {
            let verification_key = VerificationKey::new(jwk(Some(key_algorithm), None)).unwrap();

            assert_eq!(
                verification_key.algorithm,
                Some(expected_algorithm),
                "{key_algorithm}"
            );
        }
    }

    #[test]
    fn should_preserve_key_id() {
        let verification_key = VerificationKey::new(jwk(Some("RS256"), Some("key-1"))).unwrap();

        assert_eq!(verification_key.kid.as_deref(), Some("key-1"));
    }

    #[test]
    fn should_allow_missing_key_id_and_algorithm() {
        let verification_key = VerificationKey::new(jwk(None, None)).unwrap();

        assert_eq!(verification_key.kid, None);
        assert_eq!(verification_key.algorithm, None);
    }

    #[test]
    fn should_reject_unsupported_key_algorithm() {
        let result = VerificationKey::new(jwk(Some("HS256"), None));

        assert!(matches!(result, Err(Error::InvalidJwk(_))));
    }

    #[test]
    fn should_reject_invalid_key_parameters() {
        let jwk = serde_json::from_value(json!({
            "kty": "unsupported",
            "alg": "ES256",
        }))
        .unwrap();

        let result = VerificationKey::new(jwk);

        assert!(matches!(result, Err(Error::InvalidJwk(_))));
    }
}
