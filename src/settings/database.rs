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

    #[test]
    fn should_accept_valid_username() {
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
}
