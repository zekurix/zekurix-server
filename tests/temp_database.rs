use anyhow::Result;
use postgres::{Client, NoTls};
use sqlx::migrate::MigrateDatabase;
use tracing::info;
use uuid::Uuid;

use zekurix_server::secrets::database::Secrets;
use zekurix_server::settings::database::Settings;

pub struct TempDatabase {
    settings: Settings,
    secrets: Secrets,
}

impl TempDatabase {
    pub fn new(settings: &Settings, secrets: &Secrets) -> Self {
        let mut temp_database = Self {
            settings: settings.clone(),
            secrets: secrets.clone(),
        };

        temp_database.settings.database = format!(
            "{}_{}",
            temp_database.settings.database,
            Uuid::now_v7().simple()
        );

        temp_database
    }

    pub fn database(&self) -> &str {
        &self.settings.database
    }

    pub async fn create(self) -> Result<Self> {
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.settings
                .username
                .as_deref()
                .expect("database username missing"),
            self.secrets.password().expect("database password missing"),
            self.settings.host,
            self.settings.port,
            self.settings.database,
        );

        info!(database = %self.settings.database, "running database creation");
        sqlx::Postgres::create_database(&url).await?;
        info!(database = %self.settings.database, "database creation completed");

        Ok(self)
    }

    // Drop trait is synchronous.
    // Use the synchronous `postgres` crate because SQLx database deletion is async.
    // Assumes the `postgres` database exists.
    // PostgreSQL does not allow dropping the currently connected database.
    // Connect to the administrative `postgres` database and drop the test database from there.
    fn drop_database(settings: &Settings, secrets: &Secrets) -> Result<()> {
        let config = format!(
            "host={} port={} user={} password={} dbname=postgres",
            settings.host,
            settings.port,
            settings
                .username
                .as_deref()
                .expect("database username missing"),
            secrets.password().expect("database password missing"),
        );

        info!(database = %settings.database, "running database drop");
        Client::connect(&config, NoTls)?.execute(
            &format!(
                "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
                settings.database
            ),
            &[],
        )?;
        info!(database = %settings.database, "database drop completed");

        Ok(())
    }
}

impl Drop for TempDatabase {
    fn drop(&mut self) {
        let settings = self.settings.clone();
        let secrets = self.secrets.clone();

        let _ = std::thread::spawn(move || Self::drop_database(&settings, &secrets)).join();
    }
}
