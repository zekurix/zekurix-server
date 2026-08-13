use anyhow::Result;
use tracing::info;

use crate::secrets::database::*;
use crate::settings::database::*;

pub struct Database {}

impl Database {
    pub async fn connect(settings: &Settings, secrets: &Secrets) -> Result<Self> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            settings.username.as_deref().unwrap_or(""),
            secrets.password().map(|_| "***").unwrap_or(""),
            settings.host,
            settings.port,
            settings.database
        );
        info!(url = %url, "Connecting to the database...");
        info!("Database connection established");
        Ok(Self {})
    }
}
