use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub discovery_url: Option<Url>,
    pub jwks_file: Option<PathBuf>,
    #[serde(with = "humantime_serde")]
    pub startup_retry_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub startup_retry_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub refresh_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub refresh_retry_interval: Duration,
    #[serde(with = "humantime_serde")]
    pub stale_keys_ttl: Duration,
    #[serde(with = "humantime_serde")]
    pub request_timeout: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            discovery_url: None,
            jwks_file: None,
            startup_retry_timeout: Duration::from_secs(30),
            startup_retry_interval: Duration::from_secs(1),
            refresh_interval: Duration::from_hours(1),
            refresh_retry_interval: Duration::from_mins(1),
            stale_keys_ttl: Duration::from_hours(1),
            request_timeout: Duration::from_secs(10),
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        match (self.discovery_url.is_some(), self.jwks_file.is_some()) {
            (true, false) | (false, true) => {}

            (false, false) => {
                return Err(Error::InvalidSettings {
                    setting: "authentication.provider".into(),
                    reason: "either discovery_url or jwks_file must be configured".into(),
                });
            }

            (true, true) => {
                return Err(Error::InvalidSettings {
                    setting: "authentication.provider".into(),
                    reason: "discovery_url and jwks_file cannot be configured at the same time"
                        .into(),
                });
            }
        }

        if self.startup_retry_timeout.is_zero() {
            return Err(Error::InvalidSettings {
                setting: "authentication.provider.startup_retry_timeout".into(),
                reason: "must be greater than zero".into(),
            });
        }

        if self.startup_retry_interval.is_zero() {
            return Err(Error::InvalidSettings {
                setting: "authentication.provider.startup_retry_interval".into(),
                reason: "must be greater than zero".into(),
            });
        }

        if self.startup_retry_interval > self.startup_retry_timeout {
            return Err(Error::InvalidSettings {
                setting: "authentication.provider.startup_retry_interval".into(),
                reason: "must not exceed authentication.provider.startup_retry_timeout".into(),
            });
        }

        if self.refresh_interval.is_zero() {
            return Err(Error::InvalidSettings {
                setting: "authentication.provider.refresh_interval".into(),
                reason: "must be greater than zero".into(),
            });
        }

        if self.refresh_retry_interval.is_zero() {
            return Err(Error::InvalidSettings {
                setting: "authentication.provider.refresh_retry_interval".into(),
                reason: "must be greater than zero".into(),
            });
        }

        if self.request_timeout.is_zero() {
            return Err(Error::InvalidSettings {
                setting: "authentication.provider.request_timeout".into(),
                reason: "must be greater than zero".into(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_settings() -> Settings {
        Settings {
            discovery_url: Some(
                "https://auth.example.com/.well-known/openid-configuration"
                    .parse()
                    .unwrap(),
            ),
            ..Default::default()
        }
    }

    #[test]
    fn validate_should_succeed_when_only_discovery_url_is_configured() {
        let settings = Settings {
            discovery_url: Some(
                "https://auth.example.com/.well-known/openid-configuration"
                    .parse()
                    .unwrap(),
            ),
            jwks_file: None,
            ..Default::default()
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_should_succeed_when_only_jwks_file_is_configured() {
        let settings = Settings {
            discovery_url: None,
            jwks_file: Some("tests/keys/authentication/jwks.json".into()),
            ..Default::default()
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validate_should_fail_when_neither_discovery_url_nor_jwks_file_is_configured() {
        let settings = Settings {
            discovery_url: None,
            jwks_file: None,
            ..Default::default()
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
            jwks_file: Some("tests/keys/authentication/jwks.json".into()),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_should_fail_when_startup_retry_timeout_is_zero() {
        let settings = Settings {
            startup_retry_timeout: Duration::ZERO,
            ..valid_settings()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_should_fail_when_startup_retry_interval_is_zero() {
        let settings = Settings {
            startup_retry_interval: Duration::ZERO,
            ..valid_settings()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_should_fail_when_startup_retry_interval_exceeds_startup_retry_timeout() {
        let settings = Settings {
            startup_retry_timeout: Duration::from_secs(10),
            startup_retry_interval: Duration::from_secs(11),
            ..valid_settings()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_should_succeed_when_startup_retry_interval_equals_startup_retry_timeout() {
        let settings = Settings {
            startup_retry_timeout: Duration::from_secs(10),
            startup_retry_interval: Duration::from_secs(10),
            ..valid_settings()
        };
        let result = settings.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_should_fail_when_refresh_interval_is_zero() {
        let settings = Settings {
            refresh_interval: Duration::ZERO,
            ..valid_settings()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_should_fail_when_refresh_retry_interval_is_zero() {
        let settings = Settings {
            refresh_retry_interval: Duration::ZERO,
            ..valid_settings()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn validate_should_succeed_when_stale_keys_ttl_is_zero() {
        let settings = Settings {
            stale_keys_ttl: Duration::ZERO,
            ..valid_settings()
        };
        let result = settings.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn validate_should_fail_when_request_timeout_is_zero() {
        let settings = Settings {
            request_timeout: Duration::ZERO,
            ..valid_settings()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }
}
