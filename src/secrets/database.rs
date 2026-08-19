use std::env;

use secrecy::{ExposeSecret, SecretString};

const ENV_PASSWORD: &str = "ZEKURIX_DATABASE__PASSWORD";

#[derive(Clone, Debug, Default)]
pub struct Secrets {
    password: Option<SecretString>,
}

impl Secrets {
    pub fn env_vars() -> Vec<&'static str> {
        vec![ENV_PASSWORD]
    }

    pub fn load() -> Self {
        Self {
            password: env::var(ENV_PASSWORD).ok().map(Into::into),
        }
    }

    pub fn password(&self) -> Option<&str> {
        self.password.as_ref().map(|p| p.expose_secret())
    }
}
