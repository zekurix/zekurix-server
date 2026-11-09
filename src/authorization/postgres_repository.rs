use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{error, instrument};

use crate::error::{Error, Result};

use super::{Permission, repository::AuthorizationRepository};

pub struct PostgresAuthorizationRepository {
    pool: PgPool,
}

impl PostgresAuthorizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorizationRepository for PostgresAuthorizationRepository {
    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn list_permissions(&self) -> Result<Vec<Permission>> {
        let permissions = sqlx::query_scalar::<_, String>("SELECT name FROM permissions")
            .fetch_all(&self.pool)
            .await
            .map_err(|err| {
                error!(error = ?err, "failed to list permissions");
                Error::InternalError(err.into())
            })?;

        permissions
            .into_iter()
            .map(|permission| permission.parse())
            .collect()
    }
}
