use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;
use crate::health::{HealthCheck, HealthStatus};
use crate::identity::AuthenticationService;
use crate::identity::JwksService;
use crate::settings::Settings;

use super::repositories::Repositories;
use super::stores::Stores;

pub struct Services {
    pub jwks: Arc<JwksService>,
    pub authentication: AuthenticationService,
}

impl Services {
    pub async fn new(
        settings: &Settings,
        repositories: &Repositories,
        stores: &Stores,
    ) -> Result<Self> {
        let jwks = JwksService::start(
            &settings.authentication.provider,
            stores.verification_key.clone(),
        )
        .await?;

        let authentication = AuthenticationService::new(
            &settings.authentication,
            stores.verification_key.clone(),
            repositories.identity.clone(),
            repositories.user.clone(),
        );

        Ok(Self {
            jwks,
            authentication,
        })
    }
}

#[async_trait]
impl HealthCheck for Services {
    async fn health(&self) -> HealthStatus {
        self.jwks.health().await
    }
}
