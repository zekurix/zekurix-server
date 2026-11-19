use sqlx::PgPool;

use zekurix_server::{
    authorization::{PostgresAuthorizationBootstrapRepository, PostgresAuthorizationRepository},
    user::PostgresUserRepository,
};

use crate::repositories::postgres::test_database::TestDatabase;

pub(super) struct TestHarness {
    _database: TestDatabase,
    pub(super) pool: PgPool,
    pub(super) repository: PostgresAuthorizationBootstrapRepository,
    pub(super) authorization_repository: PostgresAuthorizationRepository,
    pub(super) user_repository: PostgresUserRepository,
}

impl TestHarness {
    pub(super) async fn new() -> Self {
        let database = TestDatabase::new().await;
        let pool = database.pool.clone();
        let repository = PostgresAuthorizationBootstrapRepository::new(pool.clone());
        let authorization_repository = PostgresAuthorizationRepository::new(pool.clone());
        let user_repository = PostgresUserRepository::new(pool.clone());

        Self {
            _database: database,
            pool,
            repository,
            authorization_repository,
            user_repository,
        }
    }
}
