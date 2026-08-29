use std::str::FromStr;

use serde::Serialize;
use utoipa::ToSchema;

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, sqlx::Type, ToSchema)]
#[sqlx(transparent)]
#[schema(
    pattern = "^[A-Za-z0-9_-]{3,64}$",
    example = "alice-123"
)]
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
    fn should_implement_display() {
        let username = Username::new("alice").unwrap();

        assert_eq!(username.to_string(), "alice");
    }
}
