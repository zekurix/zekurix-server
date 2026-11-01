use std::sync::Arc;

use crate::error::{Error, Result};
use crate::settings::authentication::Settings;
use crate::user::{User, UserId, UserRepository};

use super::jwt::JwtVerifier;
use super::{Identity, IdentityRepository, Issuer, Subject, VerificationKeyStore};

pub struct AuthenticationService {
    jwt_verifier: JwtVerifier,
    identity_repository: Arc<dyn IdentityRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl AuthenticationService {
    pub fn new(
        settings: &Settings,
        key_store: Arc<VerificationKeyStore>,
        identity_repository: Arc<dyn IdentityRepository>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            jwt_verifier: JwtVerifier::new(settings, key_store),
            identity_repository,
            user_repository,
        }
    }

    pub async fn authenticate(&self, token: &str) -> Result<UserId> {
        let claims = self.jwt_verifier.verify(token).await?;
        let issuer = Issuer::new(&claims.iss).map_err(|_| Error::InvalidToken)?;
        let subject = Subject::new(&claims.sub).map_err(|_| Error::InvalidToken)?;
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
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, jwk::JwkSet};
    use mockall::predicate::eq;
    use serde_json::{Value, json};

    use crate::identity::MockIdentityRepository;
    use crate::identity::VerificationKeySet;
    use crate::user::MockUserRepository;

    use super::*;

    const ISSUER: &str = "https://auth.example.com/my-issuer";
    const AUDIENCE: &str = "zekurix";
    const SUBJECT: &str = "Alice";
    const JWKS: &str = include_str!("../../tests/keys/authentication/jwks.json");
    const PRIVATE_KEY_PATH: &str = "tests/keys/authentication/private.pem";

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn settings() -> Settings {
        Settings {
            issuers: vec![ISSUER.to_owned()],
            audience: AUDIENCE.to_owned(),
            ..Default::default()
        }
    }

    fn encoding_key() -> EncodingKey {
        let private_key = fs::read(Path::new(PRIVATE_KEY_PATH)).unwrap();

        EncodingKey::from_ec_pem(&private_key).unwrap()
    }

    fn build_valid_claims() -> Value {
        json!({
            "iss": ISSUER,
            "sub": SUBJECT,
            "aud": AUDIENCE,
            "exp": now() + 900,
            "nbf": now() - 1
        })
    }

    fn encode_claims(claims: &Value) -> String {
        encode(&Header::new(Algorithm::ES256), claims, &encoding_key()).unwrap()
    }

    fn service(
        identity_repository: MockIdentityRepository,
        user_repository: MockUserRepository,
    ) -> AuthenticationService {
        service_with_settings(&settings(), identity_repository, user_repository)
    }

    fn service_with_settings(
        settings: &Settings,
        identity_repository: MockIdentityRepository,
        user_repository: MockUserRepository,
    ) -> AuthenticationService {
        let jwks: JwkSet = serde_json::from_str(JWKS).unwrap();
        AuthenticationService::new(
            settings,
            Arc::new(VerificationKeyStore::new(VerificationKeySet::new(jwks))),
            Arc::new(identity_repository),
            Arc::new(user_repository),
        )
    }

    fn service_without_repository_expectations() -> AuthenticationService {
        service(MockIdentityRepository::new(), MockUserRepository::new())
    }

    #[tokio::test]
    async fn should_return_existing_user_id() {
        let issuer = Issuer::new(ISSUER).unwrap();
        let subject = Subject::new(SUBJECT).unwrap();
        let user_id = UserId::new();

        let identity = Identity::new(issuer.clone(), subject.clone(), user_id);

        let mut identity_repository = MockIdentityRepository::new();
        identity_repository
            .expect_find()
            .with(eq(issuer.clone()), eq(subject.clone()))
            .times(1)
            .return_once(move |_, _| Ok(identity));

        let mut user_repository = MockUserRepository::new();
        user_repository.expect_create().times(0);

        let service = service(identity_repository, user_repository);

        let result = service.find_or_create_user(issuer, subject).await.unwrap();

        assert_eq!(result, user_id);
    }

    #[tokio::test]
    async fn should_create_user_when_identity_does_not_exist() {
        let issuer = Issuer::new(ISSUER).unwrap();
        let subject = Subject::new(SUBJECT).unwrap();

        let mut identity_repository = MockIdentityRepository::new();
        identity_repository
            .expect_find()
            .with(eq(issuer.clone()), eq(subject.clone()))
            .times(1)
            .returning(|issuer, subject| {
                Err(Error::IdentityNotFound {
                    issuer: issuer.clone(),
                    subject: subject.clone(),
                })
            });

        let expected_issuer = issuer.clone();
        let expected_subject = subject.clone();

        let expected_user = User::new();
        let expected_user_id = expected_user.id;

        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_create()
            .withf(move |user, identity| {
                identity.issuer == expected_issuer
                    && identity.subject == expected_subject
                    && identity.user_id == user.id
            })
            .times(1)
            .return_once(move |_user, _identity| Ok(expected_user));

        let service = service(identity_repository, user_repository);

        let user_id = service.find_or_create_user(issuer, subject).await.unwrap();

        assert_eq!(user_id, expected_user_id);
    }

    #[tokio::test]
    async fn should_authenticate_existing_identity() {
        let user_id = UserId::new();
        let token = encode_claims(&build_valid_claims());

        let mut identity_repository = MockIdentityRepository::new();
        identity_repository
            .expect_find()
            .withf(|issuer, subject| issuer.as_str() == ISSUER && subject.as_str() == SUBJECT)
            .times(1)
            .return_once(move |issuer, subject| {
                Ok(Identity::new(issuer.clone(), subject.clone(), user_id))
            });

        let mut user_repository = MockUserRepository::new();
        user_repository.expect_create().times(0);

        let service = service(identity_repository, user_repository);

        let result = service.authenticate(&token).await.unwrap();

        assert_eq!(result, user_id);
    }

    #[tokio::test]
    async fn should_provision_user_for_unknown_identity() {
        let token = encode_claims(&build_valid_claims());

        let mut identity_repository = MockIdentityRepository::new();
        identity_repository
            .expect_find()
            .times(1)
            .returning(|issuer, subject| {
                Err(Error::IdentityNotFound {
                    issuer: issuer.clone(),
                    subject: subject.clone(),
                })
            });

        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_create()
            .withf(|user, identity| {
                identity.issuer.as_str() == ISSUER
                    && identity.subject.as_str() == SUBJECT
                    && identity.user_id == user.id
            })
            .times(1)
            .returning(|user, _| Ok(user));

        let service = service(identity_repository, user_repository);

        let result = service.authenticate(&token).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_reject_invalid_token_without_querying_repository() {
        let mut identity_repository = MockIdentityRepository::new();
        identity_repository.expect_find().times(0);

        let mut user_repository = MockUserRepository::new();
        user_repository.expect_create().times(0);

        let service = service(identity_repository, user_repository);

        let result = service.authenticate("invalid-token").await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_empty_issuer() {
        let mut claims = build_valid_claims();
        claims["iss"] = json!("");

        let token = encode_claims(&claims);
        let service = service_without_repository_expectations();

        let result = service.authenticate(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_issuer_exceeding_maximum_length() {
        let issuer = "a".repeat(1025);
        let mut claims = build_valid_claims();
        claims["iss"] = json!(&issuer);

        let token = encode_claims(&claims);
        let mut settings = settings();
        settings.issuers = vec![issuer];
        let service = service_with_settings(
            &settings,
            MockIdentityRepository::new(),
            MockUserRepository::new(),
        );

        let result = service.authenticate(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_issuer_containing_null_character() {
        let issuer = "auth\0issuer";
        let mut claims = build_valid_claims();
        claims["iss"] = json!(issuer);

        let token = encode_claims(&claims);
        let mut settings = settings();
        settings.issuers = vec![issuer.to_owned()];
        let service = service_with_settings(
            &settings,
            MockIdentityRepository::new(),
            MockUserRepository::new(),
        );

        let result = service.authenticate(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_empty_subject() {
        let mut claims = build_valid_claims();
        claims["sub"] = json!("");

        let token = encode_claims(&claims);
        let service = service_without_repository_expectations();

        let result = service.authenticate(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_subject_containing_null_character() {
        let mut claims = build_valid_claims();
        claims["sub"] = json!("Alice\0Bob");

        let token = encode_claims(&claims);
        let service = service_without_repository_expectations();

        let result = service.authenticate(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_reject_subject_exceeding_maximum_length() {
        let mut claims = build_valid_claims();
        claims["sub"] = json!("a".repeat(1025));

        let token = encode_claims(&claims);
        let service = service_without_repository_expectations();

        let result = service.authenticate(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_propagate_identity_repository_error() {
        let token = encode_claims(&build_valid_claims());

        let mut identity_repository = MockIdentityRepository::new();
        identity_repository
            .expect_find()
            .times(1)
            .returning(|_, _| Err(Error::InvalidToken));

        let mut user_repository = MockUserRepository::new();
        user_repository.expect_create().times(0);

        let service = service(identity_repository, user_repository);

        let result = service.authenticate(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }

    #[tokio::test]
    async fn should_propagate_user_creation_error() {
        let token = encode_claims(&build_valid_claims());

        let mut identity_repository = MockIdentityRepository::new();
        identity_repository
            .expect_find()
            .times(1)
            .returning(|issuer, subject| {
                Err(Error::IdentityNotFound {
                    issuer: issuer.clone(),
                    subject: subject.clone(),
                })
            });

        let mut user_repository = MockUserRepository::new();
        user_repository
            .expect_create()
            .times(1)
            .returning(|_, _| Err(Error::InvalidToken));

        let service = service(identity_repository, user_repository);

        let result = service.authenticate(&token).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }
}
