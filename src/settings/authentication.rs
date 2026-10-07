use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub issuer: String,
    pub audience: String,
    #[serde(with = "humantime_serde")]
    pub clock_skew: Duration,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            issuer: "".to_string(),
            audience: "".to_string(),
            clock_skew: Duration::from_secs(30),
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        if self.issuer.trim().is_empty() {
            return Err(Error::InvalidSettings {
                setting: "authentication.issuer".into(),
                reason: "cannot be empty".into(),
            });
        }

        if self.audience.trim().is_empty() {
            return Err(Error::InvalidSettings {
                setting: "authentication.audience".into(),
                reason: "cannot be empty".into(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn should_accept_static_issuer_audience() {
        let settings = Settings {
            issuer: "https://auth.example.com".to_string(),
            audience: "zekurix".to_string(),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_empty_issuer() {
        let settings = Settings {
            issuer: "".to_string(),
            audience: "zekurix".to_string(),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn should_reject_only_spaces_issuer() {
        let settings = Settings {
            issuer: "   ".to_string(),
            audience: "zekurix".to_string(),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    proptest! {
        #[test]
        fn should_accept_valid_issuer(issuer in r".*\S.*") {
            let settings = Settings {
                issuer: issuer.to_string(),
                audience: "zekurix".to_string(),
                ..Default::default()
            };
            let result = settings.validate();

            prop_assert!(result.is_ok());
        }
    }

    proptest! {
        #[test]
        fn should_reject_blank_issuer(issuer in r"\s*") {
            let settings = Settings {
                issuer: issuer.to_string(),
                audience: "zekurix".to_string(),
                ..Default::default()
            };
            let result = settings.validate();

            let is_err = matches!(result, Err(Error::InvalidSettings { .. }));
            prop_assert!(is_err);
        }
    }

    #[test]
    fn should_reject_empty_audience() {
        let settings = Settings {
            issuer: "https://auth.example.com".to_string(),
            audience: "".to_string(),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn should_reject_only_spaces_audience() {
        let settings = Settings {
            issuer: "https://auth.example.com".to_string(),
            audience: "   ".to_string(),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    proptest! {
        #[test]
        fn should_accept_valid_audience(audience in r".*\S.*") {
            let settings = Settings {
                issuer: "https://auth.example.com".to_string(),
                audience: audience.to_string(),
                ..Default::default()
            };
            let result = settings.validate();

            prop_assert!(result.is_ok());
        }
    }

    proptest! {
        #[test]
        fn should_reject_blank_audience(audience in r"\s*") {
            let settings = Settings {
                issuer: "https://auth.example.com".to_string(),
                audience: audience.to_string(),
                ..Default::default()
            };
            let result = settings.validate();

            let is_err = matches!(result, Err(Error::InvalidSettings { .. }));
            prop_assert!(is_err);
        }
    }
}
