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
            Algorithm::EdDsa => "EdDSA",
        })
    }
}

impl From<Algorithm> for jsonwebtoken::Algorithm {
    fn from(algorithm: Algorithm) -> Self {
        match algorithm {
            Algorithm::Rs256 => Self::RS256,
            Algorithm::Rs384 => Self::RS384,
            Algorithm::Rs512 => Self::RS512,
            Algorithm::Ps256 => Self::PS256,
            Algorithm::Ps384 => Self::PS384,
            Algorithm::Ps512 => Self::PS512,
            Algorithm::Es256 => Self::ES256,
            Algorithm::Es384 => Self::ES384,
            Algorithm::EdDsa => Self::EdDSA,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub issuers: Vec<String>,
    pub audience: String,
    #[serde(with = "humantime_serde")]
    pub clock_skew: Duration,
    pub allowed_algorithms: Vec<Algorithm>,
    pub provider: authentication_provider::Settings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            issuers: Vec::new(),
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
        if self.issuers.is_empty() {
            return Err(Error::InvalidSettings {
                setting: "authentication.issuers".into(),
                reason: "cannot be empty".into(),
            });
        }

        for (index, issuer) in self.issuers.iter().enumerate() {
            if issuer.trim().is_empty() {
                return Err(Error::InvalidSettings {
                    setting: "authentication.issuers".into(),
                    reason: format!("value at index {index} cannot be empty"),
                });
            }
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
            issuers: vec!["https://auth.example.com/my-issuer".to_string()],
            audience: "zekurix".to_string(),
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = settings.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_accept_multiple_issuers() {
        let settings = Settings {
            issuers: vec![
                "https://auth.example.com/my-issuer1".to_string(),
                "https://auth.example.com/my-issuer2".to_string(),
            ],
            audience: "zekurix".to_string(),
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(settings.validate().is_ok());
        assert_eq!(settings.issuers[0], "https://auth.example.com/my-issuer1");
        assert_eq!(settings.issuers[1], "https://auth.example.com/my-issuer2");
    }

    #[test]
    fn should_reject_empty_issuer() {
        let settings = Settings {
            issuers: vec![],
            audience: "zekurix".to_string(),
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
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
            issuers: vec!["   ".to_string()],
            audience: "zekurix".to_string(),
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn should_reject_invalid_issuer_in_list() {
        let settings = Settings {
            issuers: vec![
                "https://auth.example.com/my-issuer".to_string(),
                "   ".to_string(),
            ],
            audience: "zekurix".to_string(),
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
                ..Default::default()
            },
            ..Default::default()
        };

        assert!(matches!(
            settings.validate(),
            Err(Error::InvalidSettings { .. })
        ));
    }

    proptest! {
        #[test]
        fn should_accept_valid_issuer(issuer in r"[a-zA-Z0-9._:/@#$%&()\\-]{1,1024}") {
            let settings = Settings {
                issuers: vec![issuer.to_string()],
                audience: "zekurix".to_string(),
                provider: authentication_provider::Settings {
                    jwks_file: Some("tests/keys/authentication/jwks.json".into()),
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
                issuers: vec![issuer.to_string()],
                audience: "zekurix".to_string(),
                provider: authentication_provider::Settings {
                    jwks_file: Some("tests/keys/authentication/jwks.json".into()),
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
            issuers: vec!["https://auth.example.com/my-issuer".to_string()],
            audience: "".to_string(),
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
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
            issuers: vec!["https://auth.example.com/my-issuer".to_string()],
            audience: "   ".to_string(),
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
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
                issuers: vec!["https://auth.example.com/my-issuer".to_string()],
                audience: audience.to_string(),
                provider: authentication_provider::Settings {
                    jwks_file: Some("tests/keys/authentication/jwks.json".into()),
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
                issuers: vec!["https://auth.example.com/my-issuer".to_string()],
                audience: audience.to_string(),
                provider: authentication_provider::Settings {
                    jwks_file: Some("tests/keys/authentication/jwks.json".into()),
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
            issuers: vec!["https://auth.example.com/my-issuer".to_string()],
            audience: "zekurix".to_string(),
            allowed_algorithms: vec![],
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
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
            issuers: vec!["https://auth.example.com/my-issuer".to_string()],
            audience: "zekurix".to_string(),
            allowed_algorithms: vec![Algorithm::Rs256, Algorithm::Rs256],
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
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
        assert_eq!(Algorithm::EdDsa.to_string(), "EdDSA");
    }

    #[test]
    fn algorithm_converts_to_jsonwebtoken_algorithm() {
        let cases = [
            (Algorithm::Rs256, jsonwebtoken::Algorithm::RS256),
            (Algorithm::Rs384, jsonwebtoken::Algorithm::RS384),
            (Algorithm::Rs512, jsonwebtoken::Algorithm::RS512),
            (Algorithm::Ps256, jsonwebtoken::Algorithm::PS256),
            (Algorithm::Ps384, jsonwebtoken::Algorithm::PS384),
            (Algorithm::Ps512, jsonwebtoken::Algorithm::PS512),
            (Algorithm::Es256, jsonwebtoken::Algorithm::ES256),
            (Algorithm::Es384, jsonwebtoken::Algorithm::ES384),
            (Algorithm::EdDsa, jsonwebtoken::Algorithm::EdDSA),
        ];

        for (algorithm, expected) in cases {
            assert_eq!(jsonwebtoken::Algorithm::from(algorithm), expected);
        }
    }
}
