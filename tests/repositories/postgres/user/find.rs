use zekurix_server::{
    error::Error,
    user::{UserId, UserRepository},
};

use crate::repositories::postgres::fixtures::user_with_identity;

use super::harness::TestHarness;

#[tokio::test]
async fn should_find_user() {
    // Persist a user and verify that find returns the expected user.
    let harness = TestHarness::new().await;
    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    harness
        .repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let found = harness.repository.find(user.id).await.unwrap();

    assert_eq!(found, user);
}

#[tokio::test]
async fn should_reject_unknown_user() {
    // Look up a non-existing user ID and verify that UserNotFound is returned.
    let harness = TestHarness::new().await;
    let id = UserId::new();

    let error = harness.repository.find(id).await.unwrap_err();

    assert!(matches!(error, Error::UserNotFound(error_id) if error_id == id));
}
