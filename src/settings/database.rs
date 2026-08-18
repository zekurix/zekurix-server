use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub username: Option<String>,
    pub port: u16,
    pub host: String,
    pub database: String,
    pub max_connections: u32,
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
            migrate: false,
        }
    }
}
