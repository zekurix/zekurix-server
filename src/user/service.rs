use std::sync::Arc;

use crate::error::{Error, Result};
use crate::identity::{Identity, Issuer, Subject};

use super::{User, UserId, UserRepository};

pub struct UserService {
    allowed_issuer: Issuer,
    repository: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(allowed_issuer: Issuer, repository: Arc<dyn UserRepository>) -> Self {
        Self {
            allowed_issuer,
            repository,
        }
    }

    pub async fn find(&self, id: UserId) -> Result<User> {
        self.repository.find(id).await
    }

    pub async fn create(&self, issuer: Issuer, subject: Subject) -> Result<User> {
        if issuer != self.allowed_issuer {
            return Err(Error::InvalidIssuer(issuer.to_string()));
        }

        let user = User::new();
        let identity = Identity::new(issuer, subject, user.id);
        self.repository.create(user, identity).await
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use crate::user::MockUserRepository;

    use super::*;

    const ISSUER: &str = "https://auth.example.com/my-issuer";
    const SUBJECT: &str = "Alice";

    fn service(repository: MockUserRepository) -> UserService {
        UserService::new(Issuer::new(ISSUER).unwrap(), Arc::new(repository))
    }

    #[tokio::test]
    async fn should_return_user_found_by_repository() {
        let expected_user = User::new();
        let expected_id = expected_user.id;
        let mut repository = MockUserRepository::new();
        repository
            .expect_find()
            .with(eq(expected_id))
            .times(1)
            .return_once(move |_| Ok(expected_user));

        let user = service(repository).find(expected_id).await.unwrap();

        assert_eq!(user.id, expected_id);
    }

    #[tokio::test]
    async fn should_propagate_find_error() {
        let id = UserId::new();
        let mut repository = MockUserRepository::new();
        repository
            .expect_find()
            .with(eq(id))
            .times(1)
            .return_once(move |_| Err(Error::UserNotFound(id)));

        let result = service(repository).find(id).await;

        assert!(matches!(result, Err(Error::UserNotFound(user_id)) if user_id == id));
    }

    #[tokio::test]
    async fn should_create_user_for_allowed_issuer() {
        let issuer = Issuer::new(ISSUER).unwrap();
        let subject = Subject::new(SUBJECT).unwrap();
        let expected_issuer = issuer.clone();
        let expected_subject = subject.clone();
        let mut repository = MockUserRepository::new();
        repository
            .expect_create()
            .withf(move |user, identity| {
                identity.issuer == expected_issuer
                    && identity.subject == expected_subject
                    && identity.user_id == user.id
            })
            .times(1)
            .return_once(|user, _| Ok(user));

        let user = service(repository).create(issuer, subject).await.unwrap();

        assert_ne!(user.id, UserId::nil());
    }

    #[tokio::test]
    async fn should_reject_user_creation_for_disallowed_issuer() {
        let rejected_issuer = Issuer::new("https://auth.example.com/invalid").unwrap();
        let mut repository = MockUserRepository::new();
        repository.expect_create().times(0);

        let result = service(repository)
            .create(rejected_issuer.clone(), Subject::new(SUBJECT).unwrap())
            .await;

        assert!(
            matches!(result, Err(Error::InvalidIssuer(value)) if value == rejected_issuer.to_string())
        );
    }

    #[tokio::test]
    async fn should_propagate_create_error() {
        let issuer = Issuer::new(ISSUER).unwrap();
        let subject = Subject::new(SUBJECT).unwrap();
        let mut repository = MockUserRepository::new();
        repository
            .expect_create()
            .times(1)
            .return_once(|_, _| Err(Error::InvalidToken));

        let result = service(repository).create(issuer, subject).await;

        assert!(matches!(result, Err(Error::InvalidToken)));
    }
}
