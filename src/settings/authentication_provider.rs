use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{Error, Result};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub discovery_url: Option<Url>,
    pub jwks_file: Option<PathBuf>,
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        match (self.discovery_url.is_some(), self.jwks_file.is_some()) {
            (true, false) | (false, true) => Ok(()),

            (false, false) => Err(Error::InvalidSettings {
                setting: "authentication.provider".into(),
                reason: "either discovery_url or jwks_file must be configured".into(),
            }),

            (true, true) => Err(Error::InvalidSettings {
                setting: "authentication.provider".into(),
                reason: "discovery_url and jwks_file cannot be configured at the same time".into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_should_succeed_when_only_discovery_url_is_configured() {
        let settings = Settings {
            discovery_url: Some(
                "https://auth.example.com/.well-known/openid-configuration"
                    .parse()
                    .unwrap(),
            ),
            jwks_file: None,
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_should_succeed_when_only_jwks_file_is_configured() {
        let settings = Settings {
            discovery_url: None,
            jwks_file: Some("tests/jwks.json".into()),
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_should_fail_when_neither_discovery_url_nor_jwks_file_is_configured() {
        let settings = Settings {
            discovery_url: None,
            jwks_file: None,
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_should_fail_when_both_discovery_url_and_jwks_file_are_configured() {
        let settings = Settings {
            discovery_url: Some(
                "https://auth.example.com/.well-known/openid-configuration"
                    .parse()
                    .unwrap(),
            ),
            jwks_file: Some("tests/jwks.json".into()),
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }
}
