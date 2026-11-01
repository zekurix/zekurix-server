use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;
use crate::health::{HealthCheck, HealthStatus};
use crate::identity::{AuthenticationService, Issuer, JwksService};
use crate::openapi::OpenApiService;
use crate::settings::Settings;
use crate::user::UserService;

use super::repositories::Repositories;
use super::stores::Stores;

pub struct Services {
    pub openapi: OpenApiService,
    pub jwks: Arc<JwksService>,
    pub authentication: AuthenticationService,
    pub user: UserService,
}

impl Services {
    pub async fn new(
        settings: &Settings,
        repositories: &Repositories,
        stores: &Stores,
    ) -> Result<Self> {
        let openapi = OpenApiService::new(&settings.authentication)?;

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

        let user = UserService::new(
            Issuer::new(&settings.authentication.issuers[0])?,
            repositories.user.clone(),
        );

        Ok(Self {
            openapi,
            jwks,
            authentication,
            user,
        })
    }
}

#[async_trait]
impl HealthCheck for Services {
    async fn health(&self) -> HealthStatus {
        self.jwks.health().await
    }
}
