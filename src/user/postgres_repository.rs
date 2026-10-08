use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{error, instrument};

use crate::error::{Error, Result};
use crate::identity::Identity;

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
    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn find(&self, id: UserId) -> Result<User> {
        sqlx::query_as::<_, User>("SELECT id, username FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| {
                error!(error = ?err, "failed to select user");
                Error::InternalError(err.into())
            })?
            .ok_or(Error::UserNotFound(id))
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn create(&self, user: User, identity: Identity) -> Result<User> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            error!(error = ?err, "failed to start transaction");
            Error::InternalError(err.into())
        })?;

        let result = sqlx::query(
            "INSERT INTO users (id, username) VALUES ($1, $2) ON CONFLICT (username) DO NOTHING",
        )
        .bind(user.id)
        .bind(user.username.clone())
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!(error = ?err, "failed to insert user");
            Error::InternalError(err.into())
        })?;

        if result.rows_affected() == 0 {
            return Err(Error::UserAlreadyExists(user.username));
        }

        let result = sqlx::query(
            "INSERT INTO identities (issuer, subject, user_id) VALUES ($1, $2, $3) ON CONFLICT (issuer, subject) DO NOTHING",
        )
        .bind(identity.issuer.clone())
        .bind(identity.subject.clone())
        .bind(user.id)
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!(error = ?err, "failed to insert identity");
            Error::InternalError(err.into())
        })?;

        if result.rows_affected() == 0 {
            return Err(Error::IdentityAlreadyExists(identity));
        }

        tx.commit().await.map_err(|err| {
            error!(error = ?err, "failed to commit transaction");
            Error::InternalError(err.into())
        })?;

        Ok(user)
    }
}
