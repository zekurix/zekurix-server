use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub username: String,
    pub port: u16,
    pub host: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    #[serde(with = "humantime_serde")]
    pub acquire_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub max_lifetime: Duration,
    pub migrate: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            username: "".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "zekurix".to_string(),
            max_connections: 8,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_mins(10),
            max_lifetime: Duration::from_mins(30),
            migrate: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        if self.username.trim().is_empty() {
            return Err(Error::InvalidSettings {
                setting: "database.username".into(),
                reason: "cannot be empty".into(),
            });
        }

        if self.max_connections < self.min_connections {
            return Err(Error::InvalidSettings {
                setting: "database.max_connections".into(),
                reason: "must be greater than or equal to database.min_connections".into(),
            });
        }

        if self.max_lifetime < self.idle_timeout {
            return Err(Error::InvalidSettings {
                setting: "database.max_lifetime".into(),
                reason: "must be greater than or equal to database.idle_timeout".into(),
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
    fn should_accept_static_username() {
        let settings = Settings {
            username: "postgres".to_string(),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_empty_username() {
        let settings = Settings {
            username: "".to_string(),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    #[test]
    fn should_reject_only_spaces_username() {
        let settings = Settings {
            username: "   ".to_string(),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    proptest! {
        #[test]
        fn should_accept_valid_username(username in r".*\S.*") {
            let settings = Settings {
                username: username.to_string(),
                ..Default::default()
            };
            let result = settings.validate();

            prop_assert!(result.is_ok());
        }
    }

    proptest! {
        #[test]
        fn should_reject_blank_username(username in r"\s*") {
            let settings = Settings {
                username: username.to_string(),
                ..Default::default()
            };
            let result = settings.validate();

            let is_err = matches!(result, Err(Error::InvalidSettings { .. }));
            prop_assert!(is_err);
        }
    }

    #[test]
    fn should_accept_connection_max_greater_than_min() {
        let settings = Settings {
            username: "postgres".to_string(),
            max_connections: 5,
            min_connections: 4,
            ..Default::default()
        };
        let result = settings.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_accept_connection_max_equal_min() {
        let settings = Settings {
            username: "postgres".to_string(),
            max_connections: 5,
            min_connections: 5,
            ..Default::default()
        };
        let result = settings.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_reject_connection_max_lesser_than_min() {
        let settings = Settings {
            username: "postgres".to_string(),
            max_connections: 5,
            min_connections: 6,
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    proptest! {
        #[test]
        fn should_verify_connection_max_greater_or_equal_than_min(max in 0..1000u32, min in 0..1000u32) {
            let settings = Settings {
                username: "postgres".to_string(),
                max_connections: max,
                min_connections: min,
                ..Default::default()
            };
            let result = settings.validate();

            prop_assert_eq!(result.is_ok(), max >= min);
        }
    }

    #[test]
    fn should_accept_max_lifetime_greater_than_idle_timeout() {
        let settings = Settings {
            username: "postgres".to_string(),
            max_lifetime: Duration::from_mins(15),
            idle_timeout: Duration::from_mins(14),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_accept_max_lifetime_equal_idle_timeout() {
        let settings = Settings {
            username: "postgres".to_string(),
            max_lifetime: Duration::from_mins(15),
            idle_timeout: Duration::from_mins(15),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(result.is_ok());
    }

    #[test]
    fn should_accept_max_lifetime_lesser_than_idle_timeout() {
        let settings = Settings {
            username: "postgres".to_string(),
            max_lifetime: Duration::from_mins(15),
            idle_timeout: Duration::from_mins(16),
            ..Default::default()
        };
        let result = settings.validate();

        assert!(matches!(result, Err(Error::InvalidSettings { .. })));
    }

    proptest! {
        #[test]
        fn should_verify_max_lifetime_greater_or_equal_than_idle_timeout(max_lifetime in 0..1000u64, idle_timeout in 0..1000u64) {
            let settings = Settings {
                username: "postgres".to_string(),
                max_lifetime: Duration::from_mins(max_lifetime),
                idle_timeout: Duration::from_mins(idle_timeout),
                ..Default::default()
            };
            let result = settings.validate();

            prop_assert_eq!(result.is_ok(), max_lifetime >= idle_timeout);
        }
    }
}
