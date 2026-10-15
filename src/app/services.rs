use crate::identity::AuthenticationService;
use crate::settings::Settings;

use super::repositories::Repositories;

pub struct Services {
    pub authentication: AuthenticationService,
}

impl Services {
    pub fn new(settings: &Settings, repositories: &Repositories) -> Self {
        Self {
            authentication: AuthenticationService::new(
                &settings.authentication,
                repositories.identity.clone(),
                repositories.user.clone(),
            ),
        }
    }
}
