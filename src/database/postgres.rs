use anyhow::Result;
use sqlx::{PgPool, postgres::PgConnectOptions, postgres::PgPoolOptions};
use tracing::info;

use crate::secrets::database::Secrets;
use crate::settings::database::Settings;

pub struct PostgresDatabase {
    pub pool: PgPool,
}

impl PostgresDatabase {
    async fn connect(settings: &Settings, secrets: &Secrets) -> Result<PgPool> {
        info!("connecting to PostgreSQL database");
        let mut options = PgConnectOptions::new()
            .host(&settings.host)
            .port(settings.port)
            .database(&settings.database)
            .password(secrets.password());

        if let Some(ref username) = settings.username {
            options = options.username(username);
        }

        let pool = PgPoolOptions::new()
            .max_connections(settings.max_connections)
            .min_connections(settings.min_connections)
            .acquire_timeout(settings.acquire_timeout)
            .idle_timeout(settings.idle_timeout)
            .max_lifetime(settings.max_lifetime)
            .connect_with(options)
            .await?;
        info!("PostgreSQL database connection established");

        Ok(pool)
    }

    async fn migrate(&self) -> Result<()> {
        info!("running PostgreSQL database migrations");
        sqlx::migrate!().run(&self.pool).await?;
        info!("PostgreSQL database migrations completed");

        Ok(())
    }

    pub async fn init(settings: &Settings, secrets: &Secrets) -> Result<Self> {
        let postgres_database = Self {
            pool: Self::connect(settings, secrets).await?,
        };

        if settings.migrate {
            postgres_database.migrate().await?;
        }

        Ok(postgres_database)
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
