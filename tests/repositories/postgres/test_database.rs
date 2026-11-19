use dotenvy::dotenv;
use sqlx::PgPool;

use zekurix_server::database::postgres::PostgresDatabase;
use zekurix_server::secrets::Secrets;

use crate::common::TempDatabase;
use crate::input::settings::fixtures::test_settings;

pub(super) struct TestDatabase {
    _temp_database: TempDatabase,
    pub(super) pool: PgPool,
}

impl TestDatabase {
    pub(super) async fn new() -> Self {
        dotenv().ok();

        let mut settings = test_settings();
        let secrets = Secrets::load().unwrap();

        let temp_database = TempDatabase::new(&settings.database, &secrets.database)
            .create()
            .await
            .unwrap();

        settings.database.database = temp_database.database().to_string();

        let postgres = PostgresDatabase::init(&settings.database, &secrets.database)
            .await
            .unwrap();

        Self {
            _temp_database: temp_database,
            pool: postgres.pool,
        }
    }
}
