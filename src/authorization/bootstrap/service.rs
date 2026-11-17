use std::sync::Arc;

use crate::authorization::RoleName;
use crate::error::{Error, Result};
use crate::identity::{Identity, Issuer, Subject};
use crate::settings::BootstrapSettings;
use crate::user::User;

use super::repository::AuthorizationBootstrapRepository;

const BOOTSTRAP_NAME: &str = "authorization";
const SUPER_ADMIN_ROLE_NAME: &str = "super-admin";

pub struct AuthorizationBootstrapService {
    settings: BootstrapSettings,
    repository: Arc<dyn AuthorizationBootstrapRepository>,
}

impl AuthorizationBootstrapService {
    pub fn new(
        settings: &BootstrapSettings,
        repository: Arc<dyn AuthorizationBootstrapRepository>,
    ) -> Self {
        Self {
            settings: settings.clone(),
            repository,
        }
    }

    pub async fn bootstrap(&self) -> Result<()> {
        let mut tx = self.repository.begin().await?;

        // The bootstrap was already completed by this or another instance.
        if tx.is_completed(BOOTSTRAP_NAME).await? {
            return Ok(());
        }

        // A super-admin must be configured for the initial bootstrap.
        let super_admin =
            self.settings
                .super_admin
                .as_ref()
                .ok_or_else(|| Error::InvalidSettings {
                    setting: "authorization.bootstrap.super_admin".into(),
                    reason: "must be configured for the initial authorization bootstrap".into(),
                })?;

        let user = User::new();
        let identity = Identity::new(
            Issuer::new(&super_admin.issuer)?,
            Subject::new(&super_admin.subject)?,
            user.id,
        );
        let role_name = RoleName::new(SUPER_ADMIN_ROLE_NAME)?;

        tx.create_super_admin(user, identity, role_name).await?;
        tx.mark_completed(BOOTSTRAP_NAME).await?;
        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::repository::AuthorizationBootstrapTransaction;
    use super::*;
    use crate::authorization::bootstrap::{
        MockAuthorizationBootstrapRepository, MockAuthorizationBootstrapTransaction,
    };
    use crate::settings::SuperAdminSettings;

    fn settings(super_admin: Option<SuperAdminSettings>) -> BootstrapSettings {
        BootstrapSettings { super_admin }
    }

    fn valid_settings() -> BootstrapSettings {
        settings(Some(SuperAdminSettings {
            issuer: "https://auth.example.com/my-issuer".into(),
            subject: "admin".into(),
        }))
    }

    fn service(
        settings: BootstrapSettings,
        repository: MockAuthorizationBootstrapRepository,
    ) -> AuthorizationBootstrapService {
        AuthorizationBootstrapService::new(&settings, Arc::new(repository))
    }

    #[tokio::test]
    async fn should_skip_bootstrap_when_already_completed() {
        let mut transaction = MockAuthorizationBootstrapTransaction::new();
        transaction
            .expect_is_completed()
            .withf(|name: &str| name == BOOTSTRAP_NAME)
            .times(1)
            .return_once(|_| Ok(true));

        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository.expect_begin().times(1).return_once(move || {
            Ok(Box::new(transaction) as Box<dyn AuthorizationBootstrapTransaction>)
        });

        let result = service(settings(None), repository).bootstrap().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_propagate_begin_error() {
        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository
            .expect_begin()
            .times(1)
            .return_once(|| Err(Error::InternalError(anyhow::anyhow!("begin failed"))));

        let result = service(valid_settings(), repository).bootstrap().await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_propagate_is_completed_error() {
        let mut transaction = MockAuthorizationBootstrapTransaction::new();
        transaction
            .expect_is_completed()
            .times(1)
            .return_once(|_| Err(Error::InternalError(anyhow::anyhow!("check failed"))));

        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository.expect_begin().times(1).return_once(move || {
            Ok(Box::new(transaction) as Box<dyn AuthorizationBootstrapTransaction>)
        });

        let result = service(valid_settings(), repository).bootstrap().await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_reject_missing_super_admin() {
        let mut transaction = MockAuthorizationBootstrapTransaction::new();
        transaction
            .expect_is_completed()
            .times(1)
            .return_once(|_| Ok(false));

        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository.expect_begin().times(1).return_once(move || {
            Ok(Box::new(transaction) as Box<dyn AuthorizationBootstrapTransaction>)
        });

        let result = service(settings(None), repository).bootstrap().await;

        assert!(
            matches!(result, Err(Error::InvalidSettings { setting, .. }) if setting == "authorization.bootstrap.super_admin")
        );
    }

    #[tokio::test]
    async fn should_reject_invalid_issuer() {
        let mut transaction = MockAuthorizationBootstrapTransaction::new();
        transaction
            .expect_is_completed()
            .times(1)
            .return_once(|_| Ok(false));

        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository.expect_begin().times(1).return_once(move || {
            Ok(Box::new(transaction) as Box<dyn AuthorizationBootstrapTransaction>)
        });

        let invalid_settings = settings(Some(SuperAdminSettings {
            issuer: String::new(),
            subject: "admin".into(),
        }));

        let result = service(invalid_settings, repository).bootstrap().await;

        assert!(matches!(result, Err(Error::InvalidIssuer(_))));
    }

    #[tokio::test]
    async fn should_reject_invalid_subject() {
        let mut transaction = MockAuthorizationBootstrapTransaction::new();
        transaction
            .expect_is_completed()
            .times(1)
            .return_once(|_| Ok(false));

        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository.expect_begin().times(1).return_once(move || {
            Ok(Box::new(transaction) as Box<dyn AuthorizationBootstrapTransaction>)
        });

        let invalid_settings = settings(Some(SuperAdminSettings {
            issuer: "https://auth.example.com/my-issuer".into(),
            subject: String::new(),
        }));

        let result = service(invalid_settings, repository).bootstrap().await;

        assert!(matches!(result, Err(Error::InvalidSubject(_))));
    }

    #[tokio::test]
    async fn should_propagate_create_super_admin_error() {
        let mut transaction = MockAuthorizationBootstrapTransaction::new();
        transaction
            .expect_is_completed()
            .times(1)
            .return_once(|_| Ok(false));
        transaction
            .expect_create_super_admin()
            .times(1)
            .return_once(|_, _, _| Err(Error::InternalError(anyhow::anyhow!("create failed"))));

        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository.expect_begin().times(1).return_once(move || {
            Ok(Box::new(transaction) as Box<dyn AuthorizationBootstrapTransaction>)
        });

        let result = service(valid_settings(), repository).bootstrap().await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_propagate_mark_completed_error() {
        let mut transaction = MockAuthorizationBootstrapTransaction::new();
        transaction
            .expect_is_completed()
            .times(1)
            .return_once(|_| Ok(false));
        transaction
            .expect_create_super_admin()
            .times(1)
            .return_once(|_, _, _| Ok(()));
        transaction
            .expect_mark_completed()
            .withf(|name: &str| name == BOOTSTRAP_NAME)
            .times(1)
            .return_once(|_| Err(Error::InternalError(anyhow::anyhow!("mark failed"))));

        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository.expect_begin().times(1).return_once(move || {
            Ok(Box::new(transaction) as Box<dyn AuthorizationBootstrapTransaction>)
        });

        let result = service(valid_settings(), repository).bootstrap().await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_propagate_commit_error() {
        let mut transaction = MockAuthorizationBootstrapTransaction::new();
        transaction
            .expect_is_completed()
            .times(1)
            .return_once(|_| Ok(false));
        transaction
            .expect_create_super_admin()
            .times(1)
            .return_once(|_, _, _| Ok(()));
        transaction
            .expect_mark_completed()
            .times(1)
            .return_once(|_| Ok(()));
        transaction
            .expect_commit()
            .times(1)
            .return_once(|| Err(Error::InternalError(anyhow::anyhow!("commit failed"))));

        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository.expect_begin().times(1).return_once(move || {
            Ok(Box::new(transaction) as Box<dyn AuthorizationBootstrapTransaction>)
        });

        let result = service(valid_settings(), repository).bootstrap().await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_complete_bootstrap_successfully() {
        let mut transaction = MockAuthorizationBootstrapTransaction::new();
        transaction
            .expect_is_completed()
            .withf(|name: &str| name == BOOTSTRAP_NAME)
            .times(1)
            .return_once(|_| Ok(false));
        transaction
            .expect_create_super_admin()
            .withf(|user, identity, role_name| {
                identity.issuer.as_str() == "https://auth.example.com/my-issuer"
                    && identity.subject.as_str() == "admin"
                    && identity.user_id == user.id
                    && role_name.as_str() == SUPER_ADMIN_ROLE_NAME
            })
            .times(1)
            .return_once(|_, _, _| Ok(()));
        transaction
            .expect_mark_completed()
            .withf(|name: &str| name == BOOTSTRAP_NAME)
            .times(1)
            .return_once(|_| Ok(()));
        transaction.expect_commit().times(1).return_once(|| Ok(()));

        let mut repository = MockAuthorizationBootstrapRepository::new();
        repository.expect_begin().times(1).return_once(move || {
            Ok(Box::new(transaction) as Box<dyn AuthorizationBootstrapTransaction>)
        });

        let result = service(valid_settings(), repository).bootstrap().await;

        assert!(result.is_ok());
    }
}
