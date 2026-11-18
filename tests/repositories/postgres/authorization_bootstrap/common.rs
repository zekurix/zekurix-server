use sqlx::PgPool;

use zekurix_server::{
    authorization::{PostgresAuthorizationBootstrapRepository, PostgresAuthorizationRepository},
    user::PostgresUserRepository,
};

use crate::repositories::postgres::common::TestDatabase;

pub struct TestRepository {
    _database: TestDatabase,
    pub pool: PgPool,
    pub repository: PostgresAuthorizationBootstrapRepository,
    pub authorization_repository: PostgresAuthorizationRepository,
    pub user_repository: PostgresUserRepository,
}

impl TestRepository {
    pub async fn new() -> Self {
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
