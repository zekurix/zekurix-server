use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, jwk::JwkSet};
use serde::Serialize;

use zekurix_server::settings::authentication::Settings;

#[derive(Clone)]
pub struct TestAuth {
    pub issuer: String,
    pub audience: String,
    pub algorithm: Algorithm,
    pub kid: String,
    pub encoding_key: EncodingKey,
    pub clock_skew: Duration,
}

#[derive(Serialize)]
struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
}

impl TestAuth {
    fn read_jwks(settings: &Settings) -> JwkSet {
        let jwks_file = settings
            .provider
            .jwks_file
            .as_ref()
            .expect("JWKS file must be configured for test authentication");

        let jwks = std::fs::read_to_string(jwks_file).expect("failed to read JWKS file");

        serde_json::from_str(&jwks).expect("failed to parse JWKS file")
    }

    fn read_private_key(settings: &Settings) -> Vec<u8> {
        let jwks_file = settings
            .provider
            .jwks_file
            .as_ref()
            .expect("JWKS file must be configured for test authentication");
        let private_key_file = jwks_file.with_file_name("private.pem");
        std::fs::read(&private_key_file).expect("failed to read test private key")
    }

    fn jwk(jwks: &JwkSet) -> &jsonwebtoken::jwk::Jwk {
        jwks.keys
            .first()
            .expect("JWKS must contain at least one key")
    }

    fn algorithm(jwks: &JwkSet) -> Algorithm {
        Self::jwk(jwks)
            .common
            .key_algorithm
            .as_ref()
            .expect("JWKS key must contain an algorithm")
            .to_string()
            .parse()
            .expect("JWKS contains an unsupported signing algorithm")
    }

    fn kid(jwks: &JwkSet) -> String {
        Self::jwk(jwks)
            .common
            .key_id
            .clone()
            .expect("JWKS key must contain a key ID")
    }

    fn encoding_key(private_key: &[u8], algorithm: Algorithm) -> EncodingKey {
        match algorithm {
            Algorithm::ES256 | Algorithm::ES384 => {
                EncodingKey::from_ec_pem(private_key).expect("invalid EC private key")
            }

            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::PS256
            | Algorithm::PS384
            | Algorithm::PS512 => {
                EncodingKey::from_rsa_pem(private_key).expect("invalid RSA private key")
            }

            Algorithm::EdDSA => {
                EncodingKey::from_ed_pem(private_key).expect("invalid EdDSA private key")
            }

            _ => panic!("unsupported private key signing algorithm"),
        }
    }

    pub fn new(settings: &Settings) -> Self {
        let jwks = Self::read_jwks(settings);
        let private_key = Self::read_private_key(settings);
        let algorithm = Self::algorithm(&jwks);

        TestAuth {
            issuer: settings.issuers[0].clone(),
            audience: settings.audience.clone(),
            algorithm,
            kid: Self::kid(&jwks),
            encoding_key: Self::encoding_key(&private_key, algorithm),
            clock_skew: settings.clock_skew,
        }
    }

    pub fn token(&self, subject: &str) -> String {
        let exp = SystemTime::now()
            .checked_add(Duration::from_mins(15))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            iss: self.issuer.clone(),
            sub: subject.to_string(),
            aud: self.audience.clone(),
            exp,
        };

        let mut header = Header::new(self.algorithm);
        header.kid = Some(self.kid.clone());

        encode(&header, &claims, &self.encoding_key).expect("failed to generate test JWT")
    }

    pub fn header(&self, subject: &str) -> String {
        format!("Bearer {}", self.token(subject))
    }
}
