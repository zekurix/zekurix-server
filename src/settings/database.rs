use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub max_connections: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            name: "zekurix".to_string(),
            user: "zekurix".to_string(),
            max_connections: 8,
        }
    }
}
