use std::sync::Arc;

use crate::error::Result;
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
            repositories.identity.clone(),
            repositories.user.clone(),
            stores.verification_key.clone(),
        );

        Ok(Self {
            jwks,
            authentication,
        })
    }
}
