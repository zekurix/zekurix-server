use sqlx::PgPool;

use crate::user::postgres_repository::PostgresUserRepository;

pub struct Repositories {
    pub user: PostgresUserRepository,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            user: PostgresUserRepository::new(pool.clone()),
        }
    }
}
