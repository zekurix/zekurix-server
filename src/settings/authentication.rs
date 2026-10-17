use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

use super::authentication_provider;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Algorithm {
    #[serde(rename = "RS256")]
    Rs256,
    #[serde(rename = "RS384")]
    Rs384,
    #[serde(rename = "RS512")]
    Rs512,
    #[serde(rename = "PS256")]
    Ps256,
    #[serde(rename = "PS384")]
    Ps384,
    #[serde(rename = "PS512")]
    Ps512,
    #[serde(rename = "ES256")]
    Es256,
    #[serde(rename = "ES384")]
    Es384,
    #[serde(rename = "ES512")]
    Es512,
    #[serde(rename = "EdDSA")]
    EdDsa,
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Algorithm::Rs256 => "RS256",
            Algorithm::Rs384 => "RS384",
            Algorithm::Rs512 => "RS512",
            Algorithm::Ps256 => "PS256",
            Algorithm::Ps384 => "PS384",
            Algorithm::Ps512 => "PS512",
            Algorithm::Es256 => "ES256",
            Algorithm::Es384 => "ES384",
            Algorithm::Es512 => "ES512",
            Algorithm::EdDsa => "EdDSA",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub issuer: String,
    pub audience: String,
    #[serde(with = "humantime_serde")]
    pub clock_skew: Duration,
    pub allowed_algorithms: Vec<Algorithm>,
    pub provider: authentication_provider::Settings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            issuer: "".to_string(),
            audience: "".to_string(),
            clock_skew: Duration::from_secs(30),
            allowed_algorithms: vec![
                Algorithm::Rs256,
                Algorithm::Ps256,
                Algorithm::Es256,
                Algorithm::EdDsa,
            ],
            provider: Default::default(),
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

        if self.allowed_algorithms.is_empty() {
            return Err(Error::InvalidSettings {
                setting: "authentication.allowed_algorithms".into(),
                reason: "cannot be empty".into(),
            });
        }

        let mut seen = std::collections::HashSet::new();
        for algo in &self.allowed_algorithms {
            if !seen.insert(*algo) {
                return Err(Error::InvalidSettings {
                    setting: "authentication.allowed_algorithms".into(),
                    reason: format!("duplicate value: {algo}"),
                });
            }
        }

        self.provider.validate()?;

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
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/jwks.json".into()),
                ..Default::default()
            },
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
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/jwks.json".into()),
                ..Default::default()
            },
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
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/jwks.json".into()),
                ..Default::default()
            },
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
                provider: authentication_provider::Settings {
                    jwks_file: Some("tests/jwks.json".into()),
                    ..Default::default()
                },
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
                provider: authentication_provider::Settings {
                    jwks_file: Some("tests/jwks.json".into()),
                    ..Default::default()
                },
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
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/jwks.json".into()),
                ..Default::default()
            },
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
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/jwks.json".into()),
                ..Default::default()
            },
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
                provider: authentication_provider::Settings {
                    jwks_file: Some("tests/jwks.json".into()),
                    ..Default::default()
                },
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
                provider: authentication_provider::Settings {
                    jwks_file: Some("tests/jwks.json".into()),
                    ..Default::default()
                },
                ..Default::default()
            };
            let result = settings.validate();

            let is_err = matches!(result, Err(Error::InvalidSettings { .. }));
            prop_assert!(is_err);
        }
    }

    #[test]
    fn should_use_default_allowed_algorithms() {
        let settings = Settings::default();

        assert_eq!(
            settings.allowed_algorithms,
            vec![
                Algorithm::Rs256,
                Algorithm::Ps256,
                Algorithm::Es256,
                Algorithm::EdDsa,
            ]
        );
    }

    #[test]
    fn should_reject_empty_allowed_algorithms() {
        let settings = Settings {
            issuer: "https://auth.example.com".to_string(),
            audience: "zekurix".to_string(),
            allowed_algorithms: vec![],
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/jwks.json".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn should_reject_duplicate_allowed_algorithms() {
        let settings = Settings {
            issuer: "https://auth.example.com".to_string(),
            audience: "zekurix".to_string(),
            allowed_algorithms: vec![Algorithm::Rs256, Algorithm::Rs256],
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/jwks.json".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn test_algorithm_to_string_all_variants() {
        assert_eq!(Algorithm::Rs256.to_string(), "RS256");
        assert_eq!(Algorithm::Rs384.to_string(), "RS384");
        assert_eq!(Algorithm::Rs512.to_string(), "RS512");
        assert_eq!(Algorithm::Ps256.to_string(), "PS256");
        assert_eq!(Algorithm::Ps384.to_string(), "PS384");
        assert_eq!(Algorithm::Ps512.to_string(), "PS512");
        assert_eq!(Algorithm::Es256.to_string(), "ES256");
        assert_eq!(Algorithm::Es384.to_string(), "ES384");
        assert_eq!(Algorithm::Es512.to_string(), "ES512");
        assert_eq!(Algorithm::EdDsa.to_string(), "EdDSA");
    }
}
