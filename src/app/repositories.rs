use std::sync::Arc;

use sqlx::PgPool;

use crate::identity::{IdentityRepository, PostgresIdentityRepository};
use crate::user::{PostgresUserRepository, UserRepository};

pub struct Repositories {
    pub identity: Arc<dyn IdentityRepository>,
    pub user: Arc<dyn UserRepository>,
}

impl Repositories {
    pub fn new(pool: PgPool) -> Self {
        Self {
            identity: Arc::new(PostgresIdentityRepository::new(pool.clone())),
            user: Arc::new(PostgresUserRepository::new(pool.clone())),
        }
    }
}
