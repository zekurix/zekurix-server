use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct Username(String);

impl Username {
    pub fn new(value: &str) -> Result<Self> {
        value.parse()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Username {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Username length must be between 3 and 64 characters.
        if !(3..=64).contains(&s.len()) {
            return Err(Error::InvalidUsername(s.to_string()));
        }

        // Username may only contain ASCII alphanumeric characters, '_' and '-'.
        if !s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(Error::InvalidUsername(s.to_string()));
        }

        Ok(Self(s.to_owned()))
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        value.parse::<Username>().map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn should_accept_alice_username() {
        let username = Username::new("alice-01_2");

        assert!(username.is_ok());
        assert_eq!(username.unwrap().as_str(), "alice-01_2");
    }

    proptest! {
        #[test]
        fn should_accept_valid_username(value in "[A-Za-z0-9_-]{3,64}") {
            let username = Username::new(&value).unwrap();

            prop_assert_eq!(username.as_str(), value);
        }
    }

    #[test]
    fn should_accept_minimum_length() {
        let username = Username::new("abc");

        assert!(username.is_ok());
    }

    #[test]
    fn should_accept_maximum_length() {
        let username = Username::new(&"a".repeat(64));

        assert!(username.is_ok());
    }

    #[test]
    fn should_reject_empty_username() {
        let username = Username::new("");

        assert!(matches!(username, Err(Error::InvalidUsername(_))));
    }

    #[test]
    fn should_reject_username_shorter_than_3_characters() {
        let username = Username::new("ab");

        assert!(matches!(username, Err(Error::InvalidUsername(_))));
    }

    #[test]
    fn should_reject_username_longer_than_64_characters() {
        let username = Username::new(&"a".repeat(65));

        assert!(matches!(username, Err(Error::InvalidUsername(_))));
    }

    #[test]
    fn should_reject_space() {
        let username = Username::new("alice bob");

        assert!(matches!(username, Err(Error::InvalidUsername(_))));
    }

    #[test]
    fn should_reject_special_characters() {
        let username = Username::new("alice@bob");

        assert!(matches!(username, Err(Error::InvalidUsername(_))));
    }

    #[test]
    fn should_reject_non_ascii_characters() {
        let username = Username::new("älîce");

        assert!(matches!(username, Err(Error::InvalidUsername(_))));
    }

    proptest! {
        #[test]
        fn should_reject_username_containing_invalid_character(
            value in "[A-Za-z0-9_-]{0,32}[@./\\\\:; !?][A-Za-z0-9_-]{0,32}"
        ) {
            let username = Username::new(&value);

            prop_assert!(matches!(username, Err(Error::InvalidUsername(_))));
        }
    }

    #[test]
    fn should_deserialize_valid_username() {
        let username: Username = serde_json::from_str(r#""Alice-01_2""#).unwrap();

        assert_eq!(username.as_str(), "Alice-01_2");
    }

    #[test]
    fn should_reject_invalid_username_during_deserialization() {
        let result: std::result::Result<Username, _> = serde_json::from_str(r#""alice@bob""#);

        assert!(result.is_err());
    }

    proptest! {
        #[test]
        fn should_round_trip_valid_username(value in "[A-Za-z0-9_-]{3,64}") {
            let username = Username::new(&value).unwrap();

            let serialized = serde_json::to_string(&username).unwrap();
            let deserialized: Username = serde_json::from_str(&serialized).unwrap();

            prop_assert_eq!(deserialized, username);
        }
    }

    #[test]
    fn should_implement_display() {
        let username = Username::new("alice").unwrap();

        assert_eq!(username.to_string(), "alice");
    }
}
