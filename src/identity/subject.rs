use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct Subject(String);

impl Subject {
    pub fn new(value: &str) -> Result<Self> {
        value.parse()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Subject {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() || s.len() > 1024 {
            return Err(Error::InvalidSubject(s.to_string()));
        }

        Ok(Self(s.to_owned()))
    }
}

impl<'de> Deserialize<'de> for Subject {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        value.parse::<Subject>().map_err(D::Error::custom)
    }
}

impl std::fmt::Display for Subject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn should_accept_test_subject() {
        let subject = Subject::new("test-subject");

        assert!(subject.is_ok());
        assert_eq!(subject.unwrap().as_str(), "test-subject");
    }

    proptest! {
        #[test]
        fn should_accept_valid_subject(value in r"[a-zA-Z0-9._:/@#$%&()\\-]{1,1024}") {
            let subject = Subject::new(&value).unwrap();

            prop_assert_eq!(subject.as_str(), value);
        }
    }

    #[test]
    fn should_accept_minimum_length() {
        let subject = Subject::new("a");

        assert!(subject.is_ok());
    }

    #[test]
    fn should_accept_maximum_length() {
        let subject = Subject::new(&"a".repeat(1024));

        assert!(subject.is_ok());
    }

    #[test]
    fn should_reject_empty_subject() {
        let subject = Subject::new("");

        assert!(matches!(subject, Err(Error::InvalidSubject(_))));
    }

    #[test]
    fn should_reject_too_long_subject() {
        let subject = Subject::new(&"a".repeat(1025));

        assert!(matches!(subject, Err(Error::InvalidSubject(_))));
    }

    #[test]
    fn should_deserialize_valid_subject() {
        let subject: Subject = serde_json::from_str(r#""test-subject""#).unwrap();

        assert_eq!(subject.as_str(), "test-subject");
    }

    #[test]
    fn should_reject_invalid_subject_during_deserialization() {
        let result: std::result::Result<Subject, _> = serde_json::from_str(r#""""#);

        assert!(result.is_err());
    }

    proptest! {
        #[test]
        fn should_round_trip_valid_subject(value in r"[a-zA-Z0-9._:/@#$%&()\\-]{1,1024}") {
            let subject = Subject::new(&value).unwrap();

            let serialized = serde_json::to_string(&subject).unwrap();
            let deserialized: Subject = serde_json::from_str(&serialized).unwrap();

            prop_assert_eq!(deserialized, subject);
        }
    }

    #[test]
    fn should_implement_display() {
        let subject = Subject::new("test-subject").unwrap();

        assert_eq!(subject.to_string(), "test-subject");
    }
}
