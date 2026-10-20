use crate::identity::AuthenticationService;
use crate::identity::JwksService;
use crate::settings::Settings;

use super::repositories::Repositories;

pub struct Services {
    pub jwks: JwksService,
    pub authentication: AuthenticationService,
}

impl Services {
    pub fn new(settings: &Settings, repositories: &Repositories) -> Self {
        let jwks = JwksService::new(&settings.authentication.provider);

        let authentication = AuthenticationService::new(
            &settings.authentication,
            repositories.identity.clone(),
            repositories.user.clone(),
        );

        Self {
            jwks,
            authentication,
        }
    }
}
