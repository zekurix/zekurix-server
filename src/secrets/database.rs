use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Secrets {
    password: SecretString,
}

impl Secrets {
    pub fn password(&self) -> &str {
        self.password.expose_secret()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_accessor_returns_correct_value() {
        let password = "my_secret_123".to_string();
        let secrets = Secrets {
            password: SecretString::new(password.clone().into_boxed_str()),
        };

        assert_eq!(secrets.password(), password);
    }

    #[test]
    fn test_default_password_is_empty() {
        let secrets = Secrets::default();
        assert_eq!(secrets.password(), "");
    }

    #[test]
    fn test_password_never_exposed_in_debug() {
        let secret_values = vec![
            "password123",
            "super_secret_key_xyz",
            "admin@2024",
            "!@#$%^&*()",
        ];

        for secret in secret_values {
            let secrets = Secrets {
                password: SecretString::new(secret.to_string().into_boxed_str()),
            };
            let debug_output = format!("{:?}", secrets);

            assert!(
                !debug_output.contains(secret),
                "Secret '{}' was exposed in debug output: {}",
                secret,
                debug_output
            );
        }
    }
}
