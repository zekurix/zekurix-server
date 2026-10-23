use jsonwebtoken::{Algorithm, jwk::JwkSet};
use tracing::debug;

use super::VerificationKey;

#[derive(Debug, Clone, Default)]
pub struct VerificationKeySet {
    pub keys: Vec<VerificationKey>,
}

impl VerificationKeySet {
    pub fn new(jwks: JwkSet) -> Self {
        let total = jwks.keys.len();

        let keys = jwks
            .keys
            .into_iter()
            .filter_map(|jwk| VerificationKey::new(jwk).ok())
            .collect::<Vec<_>>();

        debug!(
            total,
            loaded = keys.len(),
            ignored = total - keys.len(),
            "verification key set created"
        );

        Self { keys }
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn find_candidates(
        &self,
        kid: Option<&str>,
        algorithm: Algorithm,
    ) -> Vec<&VerificationKey> {
        match kid {
            Some(kid) => self
                .keys
                .iter()
                .filter(|key| {
                    key.kid.as_deref() == Some(kid)
                        && (key.algorithm.is_none() || key.algorithm == Some(algorithm))
                })
                .collect(),

            None => self
                .keys
                .iter()
                .filter(|key| key.algorithm.is_none() || key.algorithm == Some(algorithm))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn jwks(value: serde_json::Value) -> JwkSet {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn should_create_empty_set_from_empty_jwks() {
        let set = VerificationKeySet::new(jwks(json!({"keys": []})));

        assert!(set.keys.is_empty());
    }

    #[test]
    fn should_create_keys_from_valid_jwks() {
        let set = VerificationKeySet::new(jwks(json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-1",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                },
                {
                    "kty": "RSA",
                    "kid": "key-2",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                }
            ]
        })));

        assert_eq!(set.len(), 2);
        assert_eq!(set.keys[0].kid.as_deref(), Some("key-1"));
        assert_eq!(set.keys[1].kid.as_deref(), Some("key-2"));
    }

    #[test]
    fn should_ignore_invalid_jwks() {
        let set = VerificationKeySet::new(jwks(json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "valid",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                },
                {
                    "kty": "RSA",
                    "kid": "unsupported",
                    "alg": "HS256",
                    "n": "AQ",
                    "e": "AQ"
                }
            ]
        })));

        assert_eq!(set.len(), 1);
        assert_eq!(set.keys[0].kid.as_deref(), Some("valid"));
    }

    #[test]
    fn should_find_candidate_by_key_id_and_algorithm() {
        let set = VerificationKeySet::new(jwks(json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-1",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                },
                {
                    "kty": "RSA",
                    "kid": "key-2",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                }
            ]
        })));

        let candidates = set.find_candidates(Some("key-1"), Algorithm::RS256);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kid.as_deref(), Some("key-1"));
    }

    #[test]
    fn should_not_find_candidate_when_key_id_does_not_match() {
        let set = VerificationKeySet::new(jwks(json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-1",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                }
            ]
        })));

        let candidates = set.find_candidates(Some("unknown"), Algorithm::RS256);

        assert!(candidates.is_empty());
    }

    #[test]
    fn should_not_find_candidate_when_algorithm_does_not_match() {
        let set = VerificationKeySet::new(jwks(json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-1",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                }
            ]
        })));

        let candidates = set.find_candidates(Some("key-1"), Algorithm::RS512);

        assert!(candidates.is_empty());
    }

    #[test]
    fn should_find_candidate_with_matching_key_id_and_no_algorithm() {
        let set = VerificationKeySet::new(jwks(json!({
        "keys": [
            {
                "kty": "RSA",
                "kid": "key-1",
                "n": "AQ",
                "e": "AQ"
            }
        ]
        })));

        let candidates = set.find_candidates(Some("key-1"), Algorithm::RS256);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kid.as_deref(), Some("key-1"));
    }

    #[test]
    fn should_find_candidates_by_algorithm_when_key_id_is_missing() {
        let set = VerificationKeySet::new(jwks(json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-1",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                },
                {
                    "kty": "RSA",
                    "kid": "key-2",
                    "alg": "RS512",
                    "n": "AQ",
                    "e": "AQ"
                }
            ]
        })));

        let candidates = set.find_candidates(None, Algorithm::RS256);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kid.as_deref(), Some("key-1"));
    }

    #[test]
    fn should_find_candidate_without_algorithm_when_key_id_is_missing() {
        let set = VerificationKeySet::new(jwks(json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-1",
                    "n": "AQ",
                    "e": "AQ"
                }
            ]
        })));

        let candidates = set.find_candidates(None, Algorithm::RS256);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kid.as_deref(), Some("key-1"));
    }

    #[test]
    fn should_find_multiple_candidates_when_key_id_is_missing() {
        let set = VerificationKeySet::new(jwks(json!({
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "key-1",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                },
                {
                    "kty": "RSA",
                    "kid": "key-2",
                    "alg": "RS256",
                    "n": "AQ",
                    "e": "AQ"
                },
                {
                    "kty": "RSA",
                    "kid": "key-3",
                    "alg": "RS512",
                    "n": "AQ",
                    "e": "AQ"
                }
            ]
        })));

        let candidates = set.find_candidates(None, Algorithm::RS256);

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].kid.as_deref(), Some("key-1"));
        assert_eq!(candidates[1].kid.as_deref(), Some("key-2"));
    }
}
