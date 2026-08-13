use anyhow::Result;
use sqlx::{postgres::PgConnectOptions, postgres::PgPoolOptions};
use tracing::info;

use crate::secrets::database::*;
use crate::settings::database::*;

pub struct Database {
    pool: sqlx::PgPool,
}

impl Database {
    pub async fn connect(settings: &Settings, secrets: &Secrets) -> Result<Self> {
        info!("Connecting to the database...");
        let mut options = PgConnectOptions::new()
            .host(&settings.host)
            .port(settings.port)
            .database(&settings.database);

        if let Some(ref username) = settings.username {
            options = options.username(username);
        }
        if let Some(password) = secrets.password() {
            options = options.password(password);
        }

        let pool = PgPoolOptions::new()
            .max_connections(settings.max_connections)
            .connect_with(options)
            .await?;
        info!("Database connection established");

        Ok(Self { pool })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
