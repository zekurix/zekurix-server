use std::sync::Arc;

use crate::error::Result;
use crate::identity::AuthenticationService;
use crate::identity::JwksService;
use crate::settings::Settings;

use super::repositories::Repositories;

pub struct Services {
    pub jwks: Arc<JwksService>,
    pub authentication: AuthenticationService,
}

impl Services {
    pub async fn new(settings: &Settings, repositories: &Repositories) -> Result<Self> {
        let jwks = JwksService::start(&settings.authentication.provider).await?;

        let authentication = AuthenticationService::new(
            &settings.authentication,
            repositories.identity.clone(),
            repositories.user.clone(),
        );

        Ok(Self {
            jwks,
            authentication,
        })
    }
}
