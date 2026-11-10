use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{error, instrument};

use crate::error::{Error, Result};
use crate::router::Pagination;

use super::{Permission, Role, RoleId, repository::AuthorizationRepository};

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
    async fn create_role(&self, role: Role) -> Result<Role> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            error!(error = ?err, "failed to begin transaction");
            Error::InternalError(err.into())
        })?;

        let result = sqlx::query(
            "INSERT INTO roles (id, name) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING",
        )
        .bind(role.id())
        .bind(role.name())
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!(error = ?err, "failed to insert role");
            Error::InternalError(err.into())
        })?;

        if result.rows_affected() == 0 {
            return Err(Error::RoleAlreadyExists(role.name()));
        }

        for permission in role.permissions() {
            sqlx::query("INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2)")
                .bind(role.id())
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

        Ok(role)
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn get_role(&self, id: RoleId) -> Result<Role> {
        let row = sqlx::query_as::<_, (String, Vec<String>)>(
            r#"
            SELECT
                r.name,
                COALESCE(
                    ARRAY_AGG(rp.permission ORDER BY rp.permission)
                        FILTER (WHERE rp.permission IS NOT NULL),
                    ARRAY[]::TEXT[]
                )
            FROM roles r
            LEFT JOIN role_permissions rp ON rp.role_id = r.id
            WHERE r.id = $1
            GROUP BY r.id, r.name
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| {
            error!(role_id = %id, error = ?err, "failed to get role");
            Error::InternalError(err.into())
        })?
        .ok_or(Error::RoleNotFound(id))?;

        let name = RoleName::new(row.0).map_err(|err| {
            error!(role_id = %id, error = ?err, "invalid role name stored in database");
            Error::InternalError(err.into())
        })?;

        let permissions = row
            .1
            .into_iter()
            .map(|permission| {
                permission.parse::<Permission>().map_err(|err| {
                    error!(
                        role_id = %id,
                        permission = %permission,
                        error = ?err,
                        "invalid permission stored in database"
                    );
                    Error::InternalError(err.into())
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Role::from_persisted(id, name, permissions))
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn list_roles(&self, pagination: Pagination) -> Result<Vec<Role>> {
        let rows = sqlx::query_as::<_, (RoleId, String, Vec<String>)>(
            r#"
            SELECT
                r.id,
                r.name,
                COALESCE(
                    ARRAY_AGG(rp.permission ORDER BY rp.permission)
                        FILTER (WHERE rp.permission IS NOT NULL),
                    ARRAY[]::TEXT[]
                )
            FROM roles r
            LEFT JOIN role_permissions rp ON rp.role_id = r.id
            GROUP BY r.id, r.name
            ORDER BY r.id
            LIMIT $1
            OFFSET $2
            "#,
        )
        .bind(i64::from(pagination.limit))
        .bind(i64::from(pagination.offset))
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            error!(error = ?err, "failed to list roles");
            Error::InternalError(err.into())
        })?;

        rows.into_iter()
            .map(|(id, name, permissions)| {
                let name = RoleName::new(name).map_err(|err| {
                    error!(
                        role_id = %id,
                        error = ?err,
                        "invalid role name stored in database"
                    );
                    Error::InternalError(err.into())
                })?;

                let permissions = permissions
                .into_iter()
                .map(|permission| {
                     permission.parse::<Permission>().map_err(|err| {
                        error!(
                            role_id = %id,
                            permission = %permission,
                            error = ?err,
                            "invalid permission stored in database"
                        );
                        Error::InternalError(err.into())
                    })
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(Role::from_persisted(id, name, permissions))
        })
        .collect()
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn update_role(&self, role: Role) -> Result<Role> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            error!(error = ?err, "failed to begin transaction");
            Error::InternalError(err.into())
        })?;

        let result = sqlx::query("UPDATE roles SET name = $2 WHERE id = $1")
            .bind(role.id())
            .bind(role.name())
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                if err
                    .as_database_error()
                    .and_then(|err| err.code())
                    .is_some_and(|code| code == "23505")
                {
                    Error::RoleAlreadyExists(role.name().clone())
                } else {
                    error!(
                        role_id = %role.id(),
                        error = ?err,
                        "failed to update role"
                    );
                    Error::InternalError(err.into())
                }
            })?;

        if result.rows_affected() == 0 {
            return Err(Error::RoleNotFound(role.id()));
        }

        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role.id())
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                error!(
                    role_id = %role.id(),
                    error = ?err,
                    "failed to delete existing role permissions"
                );
                Error::InternalError(err.into())
            })?;

        for permission in role.permissions() {
            sqlx::query(
                "INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2)",
            )
            .bind(role.id())
            .bind(permission.as_str())
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                error!(
                    role_id = %role.id(),
                    permission = %permission,
                    error = ?err,
                    "failed to insert role permission"
                );
                Error::InternalError(err.into())
            })?;
        }

        tx.commit().await.map_err(|err| {
            error!(role_id = %role.id(), error = ?err, "failed to commit transaction");
            Error::InternalError(err.into())
        })?;

        Ok(role)
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn delete_role(&self, id: RoleId) -> Result<()> {
        let result = sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                error!(role_id = %id, error = ?err, "failed to delete role");
                Error::InternalError(err.into())
            })?;

        if result.rows_affected() == 0 {
            return Err(Error::RoleNotFound(id));
        }

        Ok(())
    }
}
