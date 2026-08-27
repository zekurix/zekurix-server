use async_trait::async_trait;
use sqlx::PgPool;
use tracing::error;

use crate::error::{Error, Result};

use super::{User, UserId, repository::UserRepository};

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find(&self, id: UserId) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT id, username FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| {
                error!(error = ?err, "Database error");
                Error::InternalError
            })?
            .ok_or(Error::UserNotFound(id))
    }

    async fn create(&self, user: User) -> Result<()> {
        let result = sqlx::query(
            "INSERT INTO users (id, username) VALUES ($1, $2) ON CONFLICT (username) DO NOTHING",
        )
        .bind(user.id)
        .bind(user.username.clone())
        .execute(&self.pool)
        .await
        .map_err(|err| {
            error!(error = ?err, "Database error");
            Error::InternalError
        })?;

        if result.rows_affected() == 0 {
            return Err(Error::UserAlreadyExists(user.username));
        }

        Ok(())
    }
}
