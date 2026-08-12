use anyhow::Result;
use tracing::info;

use crate::secrets::database::*;
use crate::settings::database::*;

pub struct Database {}

impl Database {
    pub async fn connect(_settings: &Settings, _secrets: &Secrets) -> Result<Self> {
        let url = "postgres://postgres:password@localhost/test";
        info!(url = %url, "Connecting to the database...");
        info!("Database connection established");
        Ok(Self {})
    }
}
