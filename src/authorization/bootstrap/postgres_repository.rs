use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{error, instrument};

use crate::authorization::RoleName;
use crate::error::{Error, Result};
use crate::identity::Identity;
use crate::user::User;

use super::repository::{AuthorizationBootstrapRepository, AuthorizationBootstrapTransaction};

pub struct PostgresAuthorizationBootstrapRepository {
    pool: PgPool,
}

impl PostgresAuthorizationBootstrapRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct PostgresAuthorizationBootstrapTransaction {
    tx: Transaction<'static, Postgres>,
}

#[async_trait]
impl AuthorizationBootstrapRepository for PostgresAuthorizationBootstrapRepository {
    #[instrument(skip(self), level = "info", err(level = "info"))]
    async fn begin(&self) -> Result<Box<dyn AuthorizationBootstrapTransaction>> {
        let mut tx = self.pool.begin().await.map_err(|err| {
            error!(error = ?err, "failed to begin authorization bootstrap transaction");
            Error::InternalError(err.into())
        })?;

        sqlx::query(
            r#"
            SELECT pg_advisory_xact_lock(
                hashtextextended('zekurix:authorization:bootstrap', 0)
            )
            "#,
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            error!(error = ?err, "failed to acquire authorization bootstrap lock");
            Error::InternalError(err.into())
        })?;

        Ok(Box::new(PostgresAuthorizationBootstrapTransaction { tx }))
    }
}

#[async_trait]
impl AuthorizationBootstrapTransaction for PostgresAuthorizationBootstrapTransaction {
    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn is_completed(&mut self, name: &str) -> Result<bool> {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM completed_bootstraps
                WHERE name = $1
            )
            "#,
        )
        .bind(name)
        .fetch_one(&mut *self.tx)
        .await
        .map_err(|err| {
            error!(
                bootstrap = name,
                error = ?err,
                "failed to check authorization bootstrap status"
            );
            Error::InternalError(err.into())
        })
    }

    #[instrument(
        skip(self),
        fields(user_id = %user.id, issuer = %identity.issuer, subject = %identity.subject),
        level = "info",
        ret,
        err(level = "info")
    )]
    async fn create_super_admin(
        &mut self,
        user: User,
        identity: Identity,
        role_name: RoleName,
    ) -> Result<()> {
        /*
         * Create the user and identity only when the configured identity does
         * not already exist. If the identity exists, its current user is reused.
         */
        sqlx::query(
            r#"
            WITH new_user AS (
                INSERT INTO users (id)
                SELECT $3
                WHERE NOT EXISTS (
                    SELECT 1
                    FROM identities
                    WHERE issuer = $1
                    AND subject = $2
                )
                RETURNING id
            )
            INSERT INTO identities (issuer, subject, user_id)
            SELECT
            $1,
            $2,
            id
            FROM new_user
            "#,
        )
        .bind(&identity.issuer)
        .bind(&identity.subject)
        .bind(user.id)
        .execute(&mut *self.tx)
        .await
        .map_err(|err| {
            error!(
                user_id = %user.id,
                issuer = %identity.issuer,
                subject = %identity.subject,
                error = ?err,
                "failed to create super-admin user and identity"
            );
            Error::InternalError(err.into())
        })?;

        /*
         * Assign the super-admin role to the existing or newly created user.
         * The assignment is idempotent.
         */
        sqlx::query(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            SELECT
                i.user_id,
                r.id
            FROM identities AS i
            CROSS JOIN roles AS r
            WHERE i.issuer = $1
            AND i.subject = $2
            AND r.name = $3
            ON CONFLICT (user_id, role_id) DO NOTHING
            "#,
        )
        .bind(&identity.issuer)
        .bind(&identity.subject)
        .bind(&role_name)
        .execute(&mut *self.tx)
        .await
        .map_err(|err| {
            error!(
            issuer = %identity.issuer,
            subject = %identity.subject,
            role = %role_name,
            error = ?err,
            "failed to assign super-admin role"
            );
            Error::InternalError(err.into())
        })?;

        /*
         * An INSERT affecting zero rows can mean either:
         *
         * - the role was already assigned;
         * - the super-admin role does not exist.
         *
         * Verify the final state explicitly.
         */
        let assigned = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM identities AS i
                INNER JOIN user_roles AS ur
                ON ur.user_id = i.user_id
                INNER JOIN roles AS r
                ON r.id = ur.role_id
                WHERE i.issuer = $1
                AND i.subject = $2
                AND r.name = $3
            )
            "#,
        )
        .bind(&identity.issuer)
        .bind(&identity.subject)
        .bind(&role_name)
        .fetch_one(&mut *self.tx)
        .await
        .map_err(|err| {
            error!(
                issuer = %identity.issuer,
                subject = %identity.subject,
                role = %role_name,
                error = ?err,
                "failed to verify super-admin role assignment"
            );
            Error::InternalError(err.into())
        })?;

        if !assigned {
            return Err(Error::InternalError(anyhow::anyhow!(
                "failed to assign role '{role_name}' to the super-admin identity"
            )));
        }

        Ok(())
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn mark_completed(&mut self, name: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO completed_bootstraps (name)
            VALUES ($1)
            "#,
        )
        .bind(name)
        .execute(&mut *self.tx)
        .await
        .map_err(|err| {
            error!(
                bootstrap = name,
                error = ?err,
                "failed to mark authorization bootstrap as completed"
            );
            Error::InternalError(err.into())
        })?;

        Ok(())
    }

    #[instrument(skip(self), level = "info", ret, err(level = "info"))]
    async fn commit(self: Box<Self>) -> Result<()> {
        let Self { tx } = *self;

        tx.commit().await.map_err(|err| {
            error!(
                error = ?err,
                "failed to commit authorization bootstrap transaction"
            );
            Error::InternalError(err.into())
        })
    }
}
