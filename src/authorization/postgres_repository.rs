use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{error, instrument};

use crate::authorization::RoleSummary;
use crate::database::postgres::UNIQUE_VIOLATION;
use crate::error::{Error, Result};
use crate::router::Pagination;
use crate::user::UserId;

use super::{Permission, Role, RoleId, RoleName, repository::AuthorizationRepository};

pub struct PostgresAuthorizationRepository {
    pool: PgPool,
}

impl PostgresAuthorizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    fn role_from_persisted(id: RoleId, name: String, permissions: Vec<String>) -> Result<Role> {
        let name = RoleName::new(&name).map_err(|err| {
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
                        "invalid permission value stored in database"
                    );
                    Error::InternalError(err.into())
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Role::from_parts(id, name, permissions))
    }

    fn role_summary_from_persisted(id: RoleId, name: String) -> Result<RoleSummary> {
        let name = RoleName::new(&name).map_err(|err| {
            error!(
                role_id = %id,
                error = ?err,
                "invalid role name stored in database"
            );
            Error::InternalError(err.into())
        })?;

        Ok(RoleSummary::from_parts(id, name))
    }
}

#[async_trait]
impl AuthorizationRepository for PostgresAuthorizationRepository {
    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn list_permissions(&self) -> Result<Vec<Permission>> {
        let permissions =
            sqlx::query_scalar::<_, String>("SELECT name FROM permissions ORDER BY name")
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
            let (_, name, _) = role.into_parts();
            return Err(Error::RoleAlreadyExists(name));
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
        let (name, permissions) = sqlx::query_as::<_, (String, Vec<String>)>(
            r#"
            SELECT
                r.name,
                COALESCE(
                    ARRAY_AGG(rp.permission ORDER BY rp.permission)
                        FILTER (WHERE rp.permission IS NOT NULL),
                    ARRAY[]::TEXT[]
                )
            FROM roles AS r
            LEFT JOIN role_permissions AS rp ON rp.role_id = r.id
            WHERE r.id = $1
            GROUP BY r.id
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

        Self::role_from_persisted(id, name, permissions)
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
            FROM roles AS r
            LEFT JOIN role_permissions AS rp ON rp.role_id = r.id
            GROUP BY r.id
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
            .map(|(id, name, permissions)| Self::role_from_persisted(id, name, permissions))
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
                    .is_some_and(|code| code == UNIQUE_VIOLATION)
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
            sqlx::query("INSERT INTO role_permissions (role_id, permission) VALUES ($1, $2)")
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

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn get_user_roles(&self, user_id: UserId) -> Result<Vec<RoleSummary>> {
        let rows = sqlx::query_as::<_, (Option<RoleId>, Option<String>)>(
            r#"
            SELECT
                r.id,
                r.name
            FROM users AS u
            LEFT JOIN user_roles AS ur ON ur.user_id = u.id
            LEFT JOIN roles AS r ON r.id = ur.role_id
            WHERE u.id = $1
            ORDER BY r.name
        "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            error!(user_id = %user_id, error = ?err, "failed to get user roles");
            Error::InternalError(err.into())
        })?;

        if rows.is_empty() {
            return Err(Error::UserNotFound(user_id));
        }

        let mut roles = Vec::new();

        for (role_id, role_name) in rows {
            match (role_id, role_name) {
                (Some(id), Some(name)) => {
                    roles.push(Self::role_summary_from_persisted(id, name)?);
                }
                (None, None) => {
                    // The user has no roles assigned, return an empty list.
                    continue;
                }
                _ => {
                    // Never should happen, but if it does, return an internal error.
                    return Err(Error::InternalError(anyhow::anyhow!(
                        "invalid role row for user_id {user_id}"
                    )));
                }
            }
        }

        Ok(roles)
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn assign_role_to_user(&self, user_id: UserId, role_id: RoleId) -> Result<()> {
        let (user_exists, role_exists) = sqlx::query_as::<_, (bool, bool)>(
            r#"
            WITH target AS (
                SELECT
                    EXISTS (SELECT 1 FROM users WHERE id = $1) AS user_exists,
                    EXISTS (SELECT 1 FROM roles WHERE id = $2) AS role_exists
            ),
            inserted AS (
                INSERT INTO user_roles (user_id, role_id)
                SELECT $1, $2
                FROM target
                WHERE target.user_exists AND target.role_exists
                ON CONFLICT DO NOTHING
                RETURNING 1
            )
            SELECT target.user_exists, target.role_exists
            FROM target
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            error!(
                user_id = %user_id,
                role_id = %role_id,
                error = ?err,
                "failed to assign role to user"
            );
            Error::InternalError(err.into())
        })?;

        if !user_exists {
            return Err(Error::UserNotFound(user_id));
        }

        if !role_exists {
            return Err(Error::RoleNotFound(role_id));
        }

        // Assign is idempotent, so if the role was already assigned to the user, we still return Ok.
        Ok(())
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn unassign_role_from_user(&self, user_id: UserId, role_id: RoleId) -> Result<()> {
        let user_exists = sqlx::query_scalar::<_, bool>(
            r#"
            WITH deleted AS (
                DELETE FROM user_roles
                WHERE user_id = $1 AND role_id = $2
                RETURNING user_id
            )
            SELECT EXISTS (
                SELECT 1
                FROM users
                WHERE id = $1
            )
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            error!(
                user_id = %user_id,
                role_id = %role_id,
                error = ?err,
                "failed to unassign role from user"
            );
            Error::InternalError(err.into())
        })?;

        if !user_exists {
            return Err(Error::UserNotFound(user_id));
        }

        // Unassign is idempotent, so if the role was not assigned to the user, we still return Ok.
        Ok(())
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn get_user_permissions(&self, user_id: UserId) -> Result<Vec<Permission>> {
        let rows = sqlx::query_scalar::<_, Option<String>>(
            r#"
            SELECT DISTINCT rp.permission
            FROM users AS u
            LEFT JOIN user_roles AS ur ON ur.user_id = u.id
            LEFT JOIN role_permissions AS rp ON rp.role_id = ur.role_id
            WHERE u.id = $1
            ORDER BY rp.permission
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| {
            error!(
                user_id = %user_id,
                error = ?err,
                "failed to get user permissions"
            );
            Error::InternalError(err.into())
        })?;

        if rows.is_empty() {
            return Err(Error::UserNotFound(user_id));
        }

        rows.into_iter()
            .flatten()
            .map(|permission| {
                permission.parse::<Permission>().map_err(|err| {
                    error!(
                        user_id = %user_id,
                        permission = %permission,
                        error = ?err,
                        "invalid permission value stored in database"
                    );
                    Error::InternalError(err.into())
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_role_from_persisted_data() {
        let id = RoleId::new();

        let role = PostgresAuthorizationRepository::role_from_persisted(
            id,
            "reader".to_string(),
            vec!["roles:read".to_string()],
        )
        .unwrap();

        assert_eq!(role.id(), id);
        assert_eq!(role.name().as_str(), "reader");
        assert_eq!(role.permissions(), &[Permission::RolesRead]);
    }

    #[test]
    fn should_create_role_without_permissions_from_persisted_data() {
        let id = RoleId::new();

        let role =
            PostgresAuthorizationRepository::role_from_persisted(id, "reader".to_string(), vec![])
                .unwrap();

        assert_eq!(role.id(), id);
        assert_eq!(role.name().as_str(), "reader");
        assert!(role.permissions().is_empty());
    }

    #[test]
    fn should_reject_invalid_role_name_from_persisted_data() {
        let id = RoleId::new();

        let result = PostgresAuthorizationRepository::role_from_persisted(
            id,
            "".to_string(),
            vec!["roles:read".to_string()],
        );

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[test]
    fn should_reject_invalid_permission_from_persisted_data() {
        let id = RoleId::new();

        let result = PostgresAuthorizationRepository::role_from_persisted(
            id,
            "reader".to_string(),
            vec!["unknown:permission".to_string()],
        );

        assert!(matches!(result, Err(Error::InternalError(_))));
    }
}
