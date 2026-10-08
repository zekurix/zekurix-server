use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct Issuer(String);

impl Issuer {
    pub fn new(value: &str) -> Result<Self> {
        value.parse()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Issuer {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let trimmed = s.trim();

        if trimmed.is_empty() || trimmed.len() > 1024 {
            return Err(Error::InvalidIssuer(s.to_string()));
        }

        Ok(Self(trimmed.to_owned()))
    }
}

impl<'de> Deserialize<'de> for Issuer {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        value.parse::<Issuer>().map_err(D::Error::custom)
    }
}

impl std::fmt::Display for Issuer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn should_accept_test_issuer() {
        let issuer = Issuer::new("test-issuer");

        assert!(issuer.is_ok());
        assert_eq!(issuer.unwrap().as_str(), "test-issuer");
    }

    proptest! {
        #[test]
        fn should_accept_valid_issuer(value in r"[a-zA-Z0-9._:/@#$%&()\\-]{1,1024}") {
            let issuer = Issuer::new(&value).unwrap();

            prop_assert_eq!(issuer.as_str(), value);
        }
    }

    #[test]
    fn should_accept_minimum_length() {
        let issuer = Issuer::new("a");

        assert!(issuer.is_ok());
    }

    #[test]
    fn should_accept_maximum_length() {
        let issuer = Issuer::new(&"a".repeat(1024));

        assert!(issuer.is_ok());
    }

    #[test]
    fn should_reject_empty_issuer() {
        let issuer = Issuer::new("");

        assert!(matches!(issuer, Err(Error::InvalidIssuer(_))));
    }

    #[test]
    fn should_reject_too_long_issuer() {
        let issuer = Issuer::new(&"a".repeat(1025));

        assert!(matches!(issuer, Err(Error::InvalidIssuer(_))));
    }

    #[test]
    fn should_reject_spaces_only_issuer() {
        let issuer = Issuer::new("   ");

        assert!(matches!(issuer, Err(Error::InvalidIssuer(_))));
    }

    #[test]
    fn should_deserialize_valid_issuer() {
        let issuer: Issuer = serde_json::from_str(r#""test-issuer""#).unwrap();

        assert_eq!(issuer.as_str(), "test-issuer");
    }

    #[test]
    fn should_reject_invalid_issuer_during_deserialization() {
        let result: std::result::Result<Issuer, _> = serde_json::from_str(r#""   ""#);

        assert!(result.is_err());
    }

    proptest! {
        #[test]
        fn should_round_trip_valid_issuer(value in r"[a-zA-Z0-9._:/@#$%&()\\-]{1,1024}") {
            let issuer = Issuer::new(&value).unwrap();

            let serialized = serde_json::to_string(&issuer).unwrap();
            let deserialized: Issuer = serde_json::from_str(&serialized).unwrap();

            prop_assert_eq!(deserialized, issuer);
        }
    }

    #[test]
    fn should_implement_display() {
        let issuer = Issuer::new("test-issuer").unwrap();

        assert_eq!(issuer.to_string(), "test-issuer");
    }
}
