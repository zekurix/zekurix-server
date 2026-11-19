use sqlx::PgPool;

use zekurix_server::user::PostgresUserRepository;

use crate::repositories::postgres::test_database::TestDatabase;

pub(super) struct TestHarness {
    _database: TestDatabase,
    pub(super) pool: PgPool,
    pub(super) repository: PostgresUserRepository,
}

impl TestHarness {
    pub(super) async fn new() -> Self {
        let database = TestDatabase::new().await;
        let pool = database.pool.clone();
        let repository = PostgresUserRepository::new(pool.clone());

        Self {
            _database: database,
            pool,
            repository,
        }
    }
}
