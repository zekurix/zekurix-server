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
        let super_admin = self.settings.super_admin
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
