use dotenvy::dotenv;
use sqlx::PgPool;

use zekurix_server::database::postgres::PostgresDatabase;
use zekurix_server::identity::{Identity, Issuer, Subject};
use zekurix_server::secrets::Secrets;
use zekurix_server::user::User;

use crate::common::TempDatabase;
use crate::settings::fixtures::test_settings;

pub struct TestDatabase {
    _temp_database: TempDatabase,
    pub pool: PgPool,
}

impl TestDatabase {
    pub async fn new() -> Self {
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

pub fn user_with_identity(issuer: &str, subject: &str) -> (User, Identity) {
    let user = User::new();
    let identity = Identity::new(
        Issuer::new(issuer).unwrap(),
        Subject::new(subject).unwrap(),
        user.id,
    );

    (user, identity)
}
