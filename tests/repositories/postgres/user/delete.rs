use zekurix_server::{
    error::Error,
    user::{UserId, UserRepository},
};

use crate::repositories::postgres::common::user_with_identity;

use super::common::TestRepository;

#[tokio::test]
async fn should_delete_user_and_cascade_identity() {
    // Delete an existing user and verify that both the user and its identity
    // are removed through the database cascade.
    let test = TestRepository::new().await;
    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.repository
        .create(user.clone(), identity.clone())
        .await
        .unwrap();

    test.repository.delete(user.id).await.unwrap();

    let user_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
            .bind(user.id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert!(!user_exists);

    let identity_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM identities WHERE issuer = $1 AND subject = $2)",
    )
    .bind(&identity.issuer)
    .bind(&identity.subject)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert!(!identity_exists);
}

#[tokio::test]
async fn should_reject_unknown_user() {
    // Delete a non-existing user ID and verify that UserNotFound is returned.
    let test = TestRepository::new().await;
    let id = UserId::new();

    let error = test.repository.delete(id).await.unwrap_err();

    assert!(matches!(error, Error::UserNotFound(error_id) if error_id == id));
}
