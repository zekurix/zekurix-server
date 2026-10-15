use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use crate::error::{Error, Result};
use crate::settings::authentication::Settings;
use crate::user::{User, UserId, UserRepository};

use super::claims::Claims;
use super::{Identity, IdentityRepository, Issuer, Subject};

pub struct AuthenticationService {
    validation: Validation,
    decoding_key: DecodingKey,
    identity_repository: Arc<dyn IdentityRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl AuthenticationService {
    pub fn new(
        settings: &Settings,
        identity_repository: Arc<dyn IdentityRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        let secret = settings.issuer.repeat(2);

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["iss", "sub", "aud", "exp"]);
        validation.leeway = settings.clock_skew.as_secs();
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.validate_aud = true;
        validation.set_audience(std::slice::from_ref(&settings.audience));
        validation.set_issuer(std::slice::from_ref(&settings.issuer));

        Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            validation,
            identity_repository,
            user_repository,
        }
    }

    pub async fn authenticate(&self, token: &str) -> Result<UserId> {
        let claims = self.decode(token)?;
        let issuer = Issuer::new(&claims.iss)?;
        let subject = Subject::new(&claims.sub)?;
        self.find_or_create_user(issuer, subject).await
    }

    async fn find_or_create_user(&self, issuer: Issuer, subject: Subject) -> Result<UserId> {
        match self.identity_repository.find(&issuer, &subject).await {
            Ok(identity) => Ok(identity.user_id),

            Err(Error::IdentityNotFound { .. }) => {
                let user = User::new();
                let identity = Identity::new(issuer, subject, user.id);

                self.user_repository
                    .create(user, identity)
                    .await
                    .map(|user| user.id)
            }

            Err(error) => Err(error),
        }
    }

    fn decode(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|token| token.claims)
            .map_err(|_| Error::InvalidToken)
    }
}
