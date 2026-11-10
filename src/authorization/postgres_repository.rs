use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{error, instrument};

use crate::error::{Error, Result};

use super::{Permission, Role, RoleId, RoleName, repository::AuthorizationRepository};

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
            .map(|permission| {
                permission.parse::<Permission>().map_err(|err| {
                    error!(
                        permission = %permission,
                        error = ?err,
                        "invalid permission stored in database"
                    );
                    Error::InternalError(err.into())
                })
            })
            .collect()
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn create_role(&self, name: RoleName, permissions: Vec<Permission>) -> Result<Role> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            error!(error = ?err, "failed to begin transaction");
            Error::InternalError(err.into())
        })?;

        let id = RoleId::new();
        let result = sqlx::query(
            "INSERT INTO roles (id, name) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING",
        )
        .bind(id)
        .bind(&name)
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!(error = ?err, "failed to insert role");
            Error::InternalError(err.into())
        })?;

        if result.rows_affected() == 0 {
            return Err(Error::RoleAlreadyExists(name));
        }

        for permission in &permissions {
            sqlx::query("INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2)")
                .bind(id)
                .bind(permission.as_str())
                .execute(&mut *tx)
                .await
                .map_err(|err| {
                    error!(error = ?err, "failed to insert role permission");
                    Error::InternalError(err.into())
                })?;
        }

        tx.commit().await.map_err(|err| {
            error!(error = ?err, "failed to commit transaction");
            Error::InternalError(err.into())
        })?;

        Ok(Role::from_persisted(id, name, permissions))
    }
}
