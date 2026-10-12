use sqlx::PgPool;

use crate::identity::PostgresIdentityRepository;
use crate::user::PostgresUserRepository;

pub struct Repositories {
    pub identity: PostgresIdentityRepository,
    pub user: PostgresUserRepository,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            identity: PostgresIdentityRepository::new(pool.clone()),
            user: PostgresUserRepository::new(pool.clone()),
        }
    }
}
