use std::env;

use secrecy::{ExposeSecret, SecretString};

use crate::error::{Error, Result};

const ENV_PASSWORD: &str = "ZEKURIX_DATABASE__PASSWORD";

#[derive(Clone, Debug, Default)]
pub struct Secrets {
    password: SecretString,
}

impl Secrets {
    fn new(password: &str) -> Result<Self> {
        if password.trim().is_empty() {
            return Err(Error::InvalidEnvironmentVariable(ENV_PASSWORD.to_string()));
        }

        Ok(Self {
            password: password.into(),
        })
    }

    pub fn env_vars() -> Vec<&'static str> {
        vec![ENV_PASSWORD]
    }

    pub fn load() -> Result<Self> {
        let password = env::var(ENV_PASSWORD)
            .map_err(|_| Error::MissingEnvironmentVariable(ENV_PASSWORD.to_string()))?;

        Self::new(&password)
    }

    pub fn password(&self) -> &str {
        self.password.expose_secret()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_secrets_from_valid_password() {
        let secrets = Secrets::new("password").unwrap();

        assert_eq!(secrets.password(), "password");
    }

    #[test]
    fn should_return_error_when_password_is_empty() {
        let result = Secrets::new("");

        assert!(matches!(result, Err(Error::InvalidEnvironmentVariable(_))));
    }

    #[test]
    fn should_return_error_when_password_is_only_spaces() {
        let result = Secrets::new("   ");

        assert!(matches!(result, Err(Error::InvalidEnvironmentVariable(_))));
    }
}
