use std::sync::Arc;

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use crate::error::{Error, Result};
use crate::settings::authentication::Settings;
use crate::user::{User, UserId, UserRepository};

use super::jwt::Claims;
use super::{Identity, IdentityRepository, Issuer, Subject};

const JWKS: &str = include_str!("jwks/jwks.json");

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
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_required_spec_claims(&["iss", "sub", "aud", "exp"]);
        validation.leeway = settings.clock_skew.as_secs();
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.validate_aud = true;
        validation.set_audience(std::slice::from_ref(&settings.audience));
        validation.set_issuer(std::slice::from_ref(&settings.issuer));

        let jwk_set: jsonwebtoken::jwk::JwkSet =
            serde_json::from_str(JWKS).expect("invalid jwks.json");

        let jwk = jwk_set
            .keys
            .first()
            .expect("jwks.json does not contain any key");

        let decoding_key = DecodingKey::from_jwk(jwk).expect("invalid JWK");

        Self {
            decoding_key,
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use jsonwebtoken::{EncodingKey, Header, encode};
    use mockall::predicate::eq;
    use serde_json::{Value, json};

    use crate::identity::MockIdentityRepository;
    use crate::settings::authentication_provider;
    use crate::user::MockUserRepository;

    use super::*;

    const ISSUER: &str = "test-issuer";
    const AUDIENCE: &str = "zekurix";
    const SUBJECT: &str = "Alice";

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn settings() -> Settings {
        Settings {
            issuer: ISSUER.to_owned(),
            audience: AUDIENCE.to_owned(),
            provider: authentication_provider::Settings {
                jwks_file: Some("tests/keys/authentication/jwks.json".into()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn encoding_key() -> EncodingKey {
        let private_key = fs::read(Path::new("tests/keys/authentication/private.pem")).unwrap();

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
        AuthenticationService::new(
            &settings(),
            Arc::new(identity_repository),
            Arc::new(user_repository),
        )
    }

    fn service_without_repository_expectations() -> AuthenticationService {
        service(MockIdentityRepository::new(), MockUserRepository::new())
    }

    mod decode {
        use super::*;
        use test_case::test_case;

        #[test]
        fn should_decode_valid_token() {
            let service = service_without_repository_expectations();
            let token = encode_claims(&build_valid_claims());

            let claims = service.decode(&token).unwrap();

            assert_eq!(claims.iss, ISSUER);
            assert_eq!(claims.sub, SUBJECT);
        }

        #[test]
        fn should_decode_valid_token_with_unknown_claims() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims
                .as_object_mut()
                .unwrap()
                .insert("other".to_owned(), json!("value"));
            let token = encode_claims(&claims);

            let claims = service.decode(&token).unwrap();

            assert_eq!(claims.iss, ISSUER);
            assert_eq!(claims.sub, SUBJECT);
        }

        #[test_case(""; "empty token")]
        #[test_case("invalid"; "invalid token")]
        #[test_case("header.payload"; "missing token segment")]
        #[test_case("header.payload.signature.extra"; "too many token segments")]
        #[test_case(".."; "empty segments")]
        #[test_case("header..signature"; "empty payload")]
        #[test_case("header.payload."; "empty signature")]
        fn should_reject_invalid_token(token: &str) {
            let service = service_without_repository_expectations();

            let result = service.decode(token);

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_token_with_invalid_signature() {
            let service = service_without_repository_expectations();
            let token = encode(
                &Header::new(Algorithm::HS256),
                &build_valid_claims(),
                &EncodingKey::from_secret(b"invalid-secret"),
            )
            .unwrap();

            let result = service.decode(&token);

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_token_with_invalid_algorithm() {
            let service = service_without_repository_expectations();

            let token = encode(
                &Header::new(Algorithm::HS256),
                &build_valid_claims(),
                &EncodingKey::from_secret(b"invalid-secret"),
            )
            .unwrap();

            let result = service.decode(&token);

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_token_without_issuer() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims.as_object_mut().unwrap().remove("iss");

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_token_without_subject() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims.as_object_mut().unwrap().remove("sub");

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_token_without_audience() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims.as_object_mut().unwrap().remove("aud");

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_token_without_expiration() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims.as_object_mut().unwrap().remove("exp");

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_accept_token_without_not_before() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims.as_object_mut().unwrap().remove("nbf");

            let result = service.decode(&encode_claims(&claims));

            assert!(result.is_ok());
        }

        #[test]
        fn should_reject_token_with_invalid_issuer() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims["iss"] = json!("other-issuer");

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_token_with_invalid_audience() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims["aud"] = json!("other-audience");

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_token_with_invalid_issuer_type() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims["iss"] = json!(123);

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_token_with_invalid_subject_type() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims["sub"] = json!(123);

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_reject_expired_token_outside_leeway() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims["exp"] = json!(now() - 31);

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_accept_expired_token_within_leeway() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims["exp"] = json!(now() - 29);

            let result = service.decode(&encode_claims(&claims));

            assert!(result.is_ok());
        }

        #[test]
        fn should_reject_token_used_before_nbf_outside_leeway() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims["nbf"] = json!(now() + 31);

            let result = service.decode(&encode_claims(&claims));

            assert!(matches!(result, Err(Error::InvalidToken)));
        }

        #[test]
        fn should_accept_token_used_before_nbf_within_leeway() {
            let service = service_without_repository_expectations();
            let mut claims = build_valid_claims();

            claims["nbf"] = json!(now() + 29);

            let result = service.decode(&encode_claims(&claims));

            assert!(result.is_ok());
        }
    }

    mod find_or_create_user {
        use super::*;

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
        async fn should_propagate_identity_repository_error() {
            let issuer = Issuer::new(ISSUER).unwrap();
            let subject = Subject::new(SUBJECT).unwrap();

            let mut identity_repository = MockIdentityRepository::new();
            identity_repository
                .expect_find()
                .times(1)
                .returning(|_, _| Err(Error::InternalError(anyhow::anyhow!("repository failure"))));

            let mut user_repository = MockUserRepository::new();
            user_repository.expect_create().times(0);

            let service = service(identity_repository, user_repository);

            let result = service.find_or_create_user(issuer, subject).await;

            assert!(matches!(result, Err(Error::InternalError(_))));
        }

        #[tokio::test]
        async fn should_propagate_user_creation_error() {
            let issuer = Issuer::new(ISSUER).unwrap();
            let subject = Subject::new(SUBJECT).unwrap();

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
                .returning(|_, _| Err(Error::InternalError(anyhow::anyhow!("repository failure"))));

            let service = service(identity_repository, user_repository);

            let result = service.find_or_create_user(issuer, subject).await;

            assert!(matches!(result, Err(Error::InternalError(_))));
        }
    }

    mod authenticate {
        use super::*;

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
        async fn should_reject_empty_subject() {
            let mut claims = build_valid_claims();
            claims["sub"] = json!("");

            let token = encode_claims(&claims);
            let service = service_without_repository_expectations();

            let result = service.authenticate(&token).await;

            assert!(matches!(result, Err(Error::InvalidSubject(_))));
        }

        #[tokio::test]
        async fn should_reject_subject_containing_null_character() {
            let mut claims = build_valid_claims();
            claims["sub"] = json!("Alice\0Bob");

            let token = encode_claims(&claims);
            let service = service_without_repository_expectations();

            let result = service.authenticate(&token).await;

            assert!(matches!(result, Err(Error::InvalidSubject(_))));
        }

        #[tokio::test]
        async fn should_reject_subject_exceeding_maximum_length() {
            let mut claims = build_valid_claims();
            claims["sub"] = json!("a".repeat(1025));

            let token = encode_claims(&claims);
            let service = service_without_repository_expectations();

            let result = service.authenticate(&token).await;

            assert!(matches!(result, Err(Error::InvalidSubject(_))));
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
}
