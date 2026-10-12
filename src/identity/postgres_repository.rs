use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{error, instrument};

use crate::error::{Error, Result};

use super::{Identity, Issuer, Subject, repository::IdentityRepository};

pub struct PostgresIdentityRepository {
    pool: PgPool,
}

impl PostgresIdentityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IdentityRepository for PostgresIdentityRepository {
    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn find(&self, issuer: &Issuer, subject: &Subject) -> Result<Identity> {
        sqlx::query_as::<_, Identity>(
            "SELECT issuer, subject, user_id FROM identities WHERE issuer = $1 AND subject = $2",
        )
        .bind(issuer)
        .bind(subject)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| {
            error!(error = ?err, "failed to select identities");
            Error::InternalError(err.into())
        })?
        .ok_or(Error::IdentityNotFound {
            issuer: issuer.clone(),
            subject: subject.clone(),
        })
    }
}
