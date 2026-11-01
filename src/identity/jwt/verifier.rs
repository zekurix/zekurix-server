use std::collections::HashMap;
use std::sync::Arc;

use jsonwebtoken::{Algorithm, Validation, decode, decode_header};
use tracing::trace;

use crate::error::{Error, Result};
use crate::identity::VerificationKeyStore;
use crate::settings::authentication::Settings;

use super::Claims;

pub struct JwtVerifier {
    store: Arc<VerificationKeyStore>,
    validation_map: HashMap<Algorithm, Validation>,
}

impl JwtVerifier {
    pub fn new(settings: &Settings, store: Arc<VerificationKeyStore>) -> Self {
        let algorithms = settings
            .allowed_algorithms
            .iter()
            .copied()
            .map(Into::into)
            .collect::<Vec<Algorithm>>();

        let validation_map = algorithms
            .iter()
            .copied()
            .map(|algorithm| {
                let mut validation = Validation::new(algorithm);
                validation.set_required_spec_claims(&["iss", "sub", "aud", "exp"]);
                validation.leeway = settings.clock_skew.as_secs();
                validation.validate_exp = true;
                validation.validate_nbf = true;
                validation.validate_aud = true;
                validation.set_audience(std::slice::from_ref(&settings.audience));
                validation.set_issuer(std::slice::from_ref(&settings.issuers[0]));

                (algorithm, validation)
            })
            .collect();

        Self {
            store,
            validation_map,
        }
    }

    pub async fn verify(&self, token: &str) -> Result<Claims> {
        let header = decode_header(token).map_err(|_| Error::InvalidToken)?;
        let validation = self
            .validation_map
            .get(&header.alg)
            .ok_or(Error::InvalidToken)?;
        let keys = self.store.current().await;
        let candidates = keys.find_candidates(header.kid.as_deref(), header.alg);

        for key in candidates {
            match decode::<Claims>(token, &key.decoding_key, validation) {
                Ok(token_data) => {
                    return Ok(token_data.claims);
                }
                Err(error) => trace!(
                    kid = key.kid.as_deref(),
                    error = %error,
                    "JWT verification failed for candidate key"
                ),
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, jwk::JwkSet};
    use serde_json::json;
    use test_case::test_case;

    use super::*;
    use crate::identity::{VerificationKeySet, VerificationKeyStore};
    use crate::settings::authentication::Algorithm as SettingsAlgorithm;

    const PRIVATE_KEY: &[u8] = include_bytes!("../../../tests/keys/authentication/private.pem");
    const JWKS: &str = include_str!("../../../tests/keys/authentication/jwks.json");

    fn settings_with_clock_skew(clock_skew: Duration) -> Settings {
        Settings {
            issuers: vec!["https://auth.example.com/my-issuer".into()],
            audience: "zekurix".into(),
            clock_skew,
            allowed_algorithms: vec![SettingsAlgorithm::Es256],
            ..Settings::default()
        }
    }

    fn timestamp(offset: i64) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        (now + offset) as u64
    }

    fn claims() -> serde_json::Value {
        json!({
            "iss": "https://auth.example.com/my-issuer",
            "sub": "Alice",
            "aud": "zekurix",
            "exp": timestamp(60)
        })
    }

    fn jwks() -> JwkSet {
        serde_json::from_str(JWKS).unwrap()
    }

    fn verifier(jwks: JwkSet) -> JwtVerifier {
        verifier_with_skew(jwks, Duration::ZERO)
    }

    fn verifier_with_skew(jwks: JwkSet, clock_skew: Duration) -> JwtVerifier {
        let store = Arc::new(VerificationKeyStore::new(VerificationKeySet::new(jwks)));
        JwtVerifier::new(&settings_with_clock_skew(clock_skew), store)
    }

    fn token(kid: Option<&str>) -> String {
        token_with_claims(kid, Algorithm::ES256, claims())
    }

    fn token_with_claims(
        kid: Option<&str>,
        algorithm: Algorithm,
        claims: serde_json::Value,
    ) -> String {
        let mut header = Header::new(algorithm);
        header.kid = kid.map(str::to_owned);
        encode(
            &header,
            &claims,
            &EncodingKey::from_ec_pem(PRIVATE_KEY).unwrap(),
        )
        .unwrap()
    }

    fn jwks_with_invalid_key_before_valid_key() -> JwkSet {
        let mut jwks = serde_json::to_value(jwks()).unwrap();
        jwks["keys"].as_array_mut().unwrap().insert(
            0,
            json!({
                "kid": "test",
                "crv": "P-256",
                "kty": "EC",
                "x": "AQ",
                "y": "AQ",
                "alg": "ES256",
                "use": "sig"
            }),
        );
        serde_json::from_value(jwks).unwrap()
    }

    fn jwks_with_valid_key_before_invalid_key() -> JwkSet {
        let mut jwks = serde_json::to_value(jwks_with_invalid_key_before_valid_key()).unwrap();
        jwks["keys"].as_array_mut().unwrap().swap(0, 1);
        serde_json::from_value(jwks).unwrap()
    }

    fn jwks_without_key_algorithm() -> JwkSet {
        let mut jwks = serde_json::to_value(jwks()).unwrap();
        jwks["keys"][0].as_object_mut().unwrap().remove("alg");
        serde_json::from_value(jwks).unwrap()
    }

    #[test]
    fn should_create_validation_for_each_allowed_algorithm() {
        let store = Arc::new(VerificationKeyStore::new(VerificationKeySet::new(jwks())));
        let mut settings = settings_with_clock_skew(Duration::ZERO);
        settings.allowed_algorithms = vec![
            SettingsAlgorithm::Es256,
            SettingsAlgorithm::Es384,
            SettingsAlgorithm::Rs256,
        ];
        let verifier = JwtVerifier::new(&settings, store);

        assert_eq!(verifier.validation_map.len(), 3);
        assert_eq!(
            verifier
                .validation_map
                .get(&Algorithm::ES256)
                .unwrap()
                .algorithms,
            vec![Algorithm::ES256]
        );
        assert_eq!(
            verifier
                .validation_map
                .get(&Algorithm::ES384)
                .unwrap()
                .algorithms,
            vec![Algorithm::ES384]
        );
        assert_eq!(
            verifier
                .validation_map
                .get(&Algorithm::RS256)
                .unwrap()
                .algorithms,
            vec![Algorithm::RS256]
        );
        assert!(!verifier.validation_map.contains_key(&Algorithm::EdDSA));
    }

    #[test_case(""; "empty token")]
    #[test_case("invalid"; "invalid token")]
    #[test_case("header.payload"; "missing token segment")]
    #[test_case("header.payload.signature.extra"; "too many token segments")]
    #[test_case(".."; "empty segments")]
    #[test_case("header..signature"; "empty payload")]
    #[test_case("header.payload."; "empty signature")]
    #[tokio::test]
    async fn should_reject_malformed_token(token: &str) {
        let verifier = verifier(jwks());

        let result = verifier.verify(token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_verify_valid_token() {
        let verifier = verifier(jwks());

        let claims = verifier.verify(&token(Some("test"))).await.unwrap();

        assert_eq!(claims.iss, "https://auth.example.com/my-issuer");
        assert_eq!(claims.sub, "Alice");
    }

    #[tokio::test]
    async fn should_verify_token_using_second_allowed_algorithm() {
        let store = Arc::new(VerificationKeyStore::new(VerificationKeySet::new(jwks())));
        let mut settings = settings_with_clock_skew(Duration::ZERO);
        settings.allowed_algorithms = vec![SettingsAlgorithm::Es384, SettingsAlgorithm::Es256];
        let verifier = JwtVerifier::new(&settings, store);

        let claims = verifier.verify(&token(Some("test"))).await.unwrap();

        assert_eq!(claims.sub, "Alice");
    }

    #[tokio::test]
    async fn should_verify_token_using_allowed_algorithm_from_different_family() {
        let store = Arc::new(VerificationKeyStore::new(VerificationKeySet::new(jwks())));
        let mut settings = settings_with_clock_skew(Duration::ZERO);
        settings.allowed_algorithms = vec![SettingsAlgorithm::Rs256, SettingsAlgorithm::Es256];
        let verifier = JwtVerifier::new(&settings, store);

        let claims = verifier.verify(&token(Some("test"))).await.unwrap();

        assert_eq!(claims.sub, "Alice");
    }

    #[tokio::test]
    async fn should_verify_token_without_key_id_using_first_candidate() {
        let verifier = verifier(jwks_with_valid_key_before_invalid_key());

        let claims = verifier.verify(&token(None)).await.unwrap();

        assert_eq!(claims.iss, "https://auth.example.com/my-issuer");
        assert_eq!(claims.sub, "Alice");
    }

    #[tokio::test]
    async fn should_verify_token_without_key_id_using_second_candidate() {
        let verifier = verifier(jwks_with_invalid_key_before_valid_key());

        let claims = verifier.verify(&token(None)).await.unwrap();

        assert_eq!(claims.iss, "https://auth.example.com/my-issuer");
        assert_eq!(claims.sub, "Alice");
    }

    #[tokio::test]
    async fn should_reject_token_without_key_id_after_trying_all_candidates() {
        let invalid_key: JwkSet = serde_json::from_value(json!({
            "keys": [
                {
                    "kid": "another-key",
                    "crv": "P-256",
                    "kty": "EC",
                    "x": "AQ",
                    "y": "AQ",
                    "alg": "ES256",
                    "use": "sig"
                },
                {
                    "kid": "yet-another-key",
                    "crv": "P-256",
                    "kty": "EC",
                    "x": "Ag",
                    "y": "Ag",
                    "alg": "ES256",
                    "use": "sig"
                }
            ]
        }))
        .unwrap();
        let verifier = verifier(invalid_key);

        let result = verifier.verify(&token(None)).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_token_when_key_store_is_empty() {
        let empty_jwks: JwkSet = serde_json::from_value(json!({"keys": []})).unwrap();
        let verifier = verifier(empty_jwks);

        let result = verifier.verify(&token(Some("test"))).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_token_without_matching_key_id() {
        let verifier = verifier(jwks());
        let result = verifier.verify(&token(Some("unknown"))).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_token_with_incompatible_algorithm() {
        let verifier = verifier(jwks());
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some("test".into());
        let token = encode(
            &header,
            &claims(),
            &EncodingKey::from_secret(b"not-the-ec-key"),
        )
        .unwrap();

        let result = verifier.verify(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_token_with_invalid_signature() {
        let verifier = verifier(jwks());
        let mut segments = token(Some("test"))
            .split('.')
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let replacement = if segments[2].starts_with('A') {
            'B'
        } else {
            'A'
        };
        segments[2].replace_range(..1, &replacement.to_string());
        let token = segments.join(".");

        let result = verifier.verify(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_token_without_required_issuer() {
        let verifier = verifier(jwks());
        let mut claims = claims();
        claims.as_object_mut().unwrap().remove("iss");

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_token_without_required_subject() {
        let verifier = verifier(jwks());
        let mut claims = claims();
        claims.as_object_mut().unwrap().remove("sub");

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_token_without_required_audience() {
        let verifier = verifier(jwks());
        let mut claims = claims();
        claims.as_object_mut().unwrap().remove("aud");

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_token_without_required_expiration() {
        let verifier = verifier(jwks());
        let mut claims = claims();
        claims.as_object_mut().unwrap().remove("exp");

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_expired_token_outside_clock_skew() {
        let verifier = verifier_with_skew(jwks(), Duration::from_secs(30));
        let mut claims = claims();
        claims["exp"] = json!(timestamp(-60));

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_accept_expired_token_within_clock_skew() {
        let verifier = verifier_with_skew(jwks(), Duration::from_secs(30));
        let mut claims = claims();
        claims["exp"] = json!(timestamp(-10));

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_accept_token_with_valid_not_before() {
        let verifier = verifier(jwks());
        let mut claims = claims();
        claims["nbf"] = json!(timestamp(-10));

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_reject_token_with_not_before_outside_clock_skew() {
        let verifier = verifier_with_skew(jwks(), Duration::from_secs(30));
        let mut claims = claims();
        claims["nbf"] = json!(timestamp(60));

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_accept_token_with_not_before_within_clock_skew() {
        let verifier = verifier_with_skew(jwks(), Duration::from_secs(30));
        let mut claims = claims();
        claims["nbf"] = json!(timestamp(10));

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_reject_token_with_wrong_audience() {
        let verifier = verifier(jwks());
        let mut claims = claims();
        claims["aud"] = json!("another-audience");

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_accept_token_with_audience_array_containing_configured_audience() {
        let verifier = verifier(jwks());
        let mut claims = claims();
        claims["aud"] = json!(["another-audience", "zekurix"]);

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_reject_token_with_wrong_issuer() {
        let verifier = verifier(jwks());
        let mut claims = claims();
        claims["iss"] = json!("https://auth.example.com/invalid-issuer");

        let result = verifier
            .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
            .await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_token_with_invalid_claim_types() {
        let verifier = verifier(jwks());

        for (claim, invalid_value) in [
            ("iss", json!(42)),
            ("sub", json!(42)),
            ("aud", json!(42)),
            ("exp", json!("not-a-timestamp")),
            ("nbf", json!("not-a-timestamp")),
        ] {
            let mut claims = claims();
            claims[claim] = invalid_value;
            let result = verifier
                .verify(&token_with_claims(Some("test"), Algorithm::ES256, claims))
                .await;

            assert!(
                matches!(result, Err(Error::InvalidToken)),
                "claim {claim} with an invalid type should be rejected"
            );
        }
    }

    #[tokio::test]
    async fn should_reject_token_with_algorithm_not_allowed_by_settings() {
        let store = Arc::new(VerificationKeyStore::new(VerificationKeySet::new(
            jwks_without_key_algorithm(),
        )));
        let mut settings = settings_with_clock_skew(Duration::ZERO);
        settings.allowed_algorithms = vec![SettingsAlgorithm::Es384];
        let verifier = JwtVerifier::new(&settings, store);
        let token = token(Some("test"));

        let result = verifier.verify(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }
}
