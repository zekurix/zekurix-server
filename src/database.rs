use anyhow::Result;
use sqlx::{postgres::PgConnectOptions, postgres::PgPoolOptions};
use tracing::info;

use crate::secrets::database::*;
use crate::settings::database::*;

pub struct Database {
    pub pool: sqlx::PgPool,
}

impl Database {
    async fn connect(settings: &Settings, secrets: &Secrets) -> Result<sqlx::PgPool> {
        info!("connecting to database");
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
        info!("database connection established");

        Ok(pool)
    }

    async fn migrate(&self) -> Result<()> {
        info!("running database migrations");
        sqlx::migrate!().run(&self.pool).await?;
        info!("database migrations completed");

        Ok(())
    }

    pub async fn init(settings: &Settings, secrets: &Secrets) -> Result<Self> {
        let database = Self {
            pool: Self::connect(settings, secrets).await?,
        };

        if settings.migrate {
            database.migrate().await?;
        }

        Ok(database)
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
