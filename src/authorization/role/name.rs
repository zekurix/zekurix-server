use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct RoleName(String);

impl RoleName {
    pub fn new(value: &str) -> Result<Self> {
        value.parse()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for RoleName {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 255 || s.contains('\0') {
            return Err(Error::InvalidRoleName(s.to_string()));
        }

        Ok(Self(s.to_owned()))
    }
}

impl<'de> Deserialize<'de> for RoleName {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        value.parse::<RoleName>().map_err(D::Error::custom)
    }
}

impl std::fmt::Display for RoleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn should_accept_test_role_name() {
        let name = RoleName::new("test-role");

        assert!(name.is_ok());
        assert_eq!(name.unwrap().as_str(), "test-role");
    }

    proptest! {
        #[test]
        fn should_accept_valid_role_name(value in r"[a-zA-Z0-9._:/@#$%&()\\-]{1,255}") {
            let name = RoleName::new(&value).unwrap();

            prop_assert_eq!(name.as_str(), value);
        }
    }

    #[test]
    fn should_accept_minimum_length() {
        let name = RoleName::new("a");

        assert!(name.is_ok());
    }

    #[test]
    fn should_accept_maximum_length() {
        let name = RoleName::new(&"a".repeat(255));

        assert!(name.is_ok());
    }

    #[test]
    fn should_reject_empty_role_name() {
        let name = RoleName::new("");

        assert!(matches!(name, Err(Error::InvalidRoleName(_))));
    }

    #[test]
    fn should_reject_too_long_role_name() {
        let name = RoleName::new(&"a".repeat(256));

        assert!(matches!(name, Err(Error::InvalidRoleName(_))));
    }

    #[test]
    fn should_reject_role_name_containing_null_character() {
        let name = RoleName::new("foo\0bar");

        assert!(matches!(name, Err(Error::InvalidRoleName(_))));
    }

    #[test]
    fn should_deserialize_valid_role_name() {
        let name: RoleName = serde_json::from_str(r#""test-role""#).unwrap();

        assert_eq!(name.as_str(), "test-role");
    }

    #[test]
    fn should_reject_invalid_role_name_during_deserialization() {
        let result: std::result::Result<RoleName, _> = serde_json::from_str(r#""""#);

        assert!(result.is_err());
    }

    proptest! {
        #[test]
        fn should_round_trip_valid_role_name(value in r"[a-zA-Z0-9._:/@#$%&()\\-]{1,255}") {
            let name = RoleName::new(&value).unwrap();

            let serialized = serde_json::to_string(&name).unwrap();
            let deserialized: RoleName = serde_json::from_str(&serialized).unwrap();

            prop_assert_eq!(deserialized, name);
        }
    }

    #[test]
    fn should_implement_display() {
        let name = RoleName::new("test-role").unwrap();

        assert_eq!(name.to_string(), "test-role");
    }
}
