use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub username: Option<String>,
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
            username: None,
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
