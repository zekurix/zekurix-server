use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, jwk::JwkSet};
use serde::Serialize;

use zekurix_server::settings::AuthenticationSettings;

#[derive(Debug, Serialize)]
pub(super) struct TestClaims {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) exp: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) nbf: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) other: Option<String>,
}

#[derive(Clone)]
pub(crate) struct TestAuthentication {
    issuer: String,
    audience: String,
    algorithm: Algorithm,
    kid: String,
    encoding_key: EncodingKey,
}

impl TestAuthentication {
    pub(crate) fn new(settings: &AuthenticationSettings) -> Self {
        let jwks = Self::read_jwks(settings);
        let private_key = Self::read_private_key(settings);
        let algorithm = Self::algorithm_from_jwks(&jwks);

        Self {
            issuer: settings
                .issuers
                .first()
                .expect("at least one issuer must be configured")
                .clone(),
            audience: settings.audience.clone(),
            algorithm,
            kid: Self::kid_from_jwks(&jwks),
            encoding_key: Self::encoding_key(&private_key, algorithm),
        }
    }

    pub(super) fn valid_claims(&self, subject: &str) -> TestClaims {
        let exp = SystemTime::now()
            .checked_add(Duration::from_mins(15))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        TestClaims {
            iss: Some(self.issuer.clone()),
            sub: Some(subject.to_string()),
            aud: Some(self.audience.clone()),
            exp: Some(exp),
            nbf: None,
            other: None,
        }
    }

    pub(super) fn token_with_claims(&self, claims: &TestClaims) -> String {
        let mut header = Header::new(self.algorithm);
        header.kid = Some(self.kid.clone());

        self.token_with_header(header, claims)
    }

    pub(super) fn token_with_header(&self, header: Header, claims: &TestClaims) -> String {
        encode(&header, claims, &self.encoding_key).expect("failed to generate test JWT")
    }

    pub(crate) fn token(&self, subject: &str) -> String {
        let claims = self.valid_claims(subject);

        self.token_with_claims(&claims)
    }

    pub(super) fn header_from_token(&self, token: &str) -> String {
        format!("Bearer {token}")
    }

    pub(crate) fn header(&self, subject: &str) -> String {
        let token = self.token(subject);

        self.header_from_token(&token)
    }

    pub(super) fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    pub(super) fn kid(&self) -> &str {
        &self.kid
    }

    fn read_jwks(settings: &AuthenticationSettings) -> JwkSet {
        let jwks_file = settings
            .provider
            .jwks_file
            .as_ref()
            .expect("JWKS file must be configured for test authentication");

        let jwks = std::fs::read_to_string(jwks_file).expect("failed to read JWKS file");

        serde_json::from_str(&jwks).expect("failed to parse JWKS file")
    }

    fn read_private_key(settings: &AuthenticationSettings) -> Vec<u8> {
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

    fn algorithm_from_jwks(jwks: &JwkSet) -> Algorithm {
        Self::jwk(jwks)
            .common
            .key_algorithm
            .as_ref()
            .expect("JWKS key must contain an algorithm")
            .to_string()
            .parse()
            .expect("JWKS contains an unsupported signing algorithm")
    }

    fn kid_from_jwks(jwks: &JwkSet) -> String {
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
}
