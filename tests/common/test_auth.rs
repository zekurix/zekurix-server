use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;

use zekurix_server::settings::authentication::Settings;

pub struct TestAuth {
    issuer: String,
    audience: String,
    secret: String,
}

#[derive(Serialize)]
struct Claims {
    iss: String,
    sub: String,
    aud: String,
    exp: usize,
}

impl TestAuth {
    pub fn new(settings: &Settings) -> Self {
        TestAuth {
            issuer: settings.issuer.to_string(),
            audience: settings.audience.to_string(),
            secret: settings.issuer.to_string(),
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

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .expect("failed to generate test JWT")
    }

    pub fn header(&self, subject: &str) -> String {
        format!("Bearer {}", self.token(subject))
    }
}
