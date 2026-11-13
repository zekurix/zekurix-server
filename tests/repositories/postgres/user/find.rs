use zekurix_server::{
    error::Error,
    user::{UserId, UserRepository},
};

use crate::repositories::postgres::common::user_with_identity;

use super::common::TestRepository;

#[tokio::test]
async fn should_find_user() {
    // Persist a user and verify that find returns the expected user.
    let test = TestRepository::new().await;
    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let found = test.repository.find(user.id).await.unwrap();

    assert_eq!(found, user);
}

#[tokio::test]
async fn should_reject_unknown_user() {
    // Look up a non-existing user ID and verify that UserNotFound is returned.
    let test = TestRepository::new().await;
    let id = UserId::new();

    let error = test.repository.find(id).await.unwrap_err();

    assert!(matches!(error, Error::UserNotFound(error_id) if error_id == id));
}
