use zekurix_server::error::Error;
use zekurix_server::identity::Identity;
use zekurix_server::user::{UserId, UserRepository};

use crate::repositories::postgres::common::user_with_identity;

use super::common::TestRepository;

#[tokio::test]
async fn should_create_user() {
    // Create a user with an identity and verify that both are persisted correctly.
    let test = TestRepository::new().await;
    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    let created = test
        .repository
        .create(user.clone(), identity.clone())
        .await
        .unwrap();

    assert_eq!(created.id, user.id);

    let user_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
            .bind(user.id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert!(user_exists);

    let persisted_identity = sqlx::query_as::<_, Identity>(
        "SELECT issuer, subject, user_id FROM identities WHERE issuer = $1 AND subject = $2",
    )
    .bind(&identity.issuer)
    .bind(&identity.subject)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert_eq!(persisted_identity, identity);
}

#[tokio::test]
async fn should_reject_existing_identity_and_rollback() {
    // Reuse an existing identity and verify that creation fails without persisting the new user.
    let test = TestRepository::new().await;

    let (first_user, identity) = user_with_identity("test-issuer", "test-subject");

    test.repository
        .create(first_user, identity.clone())
        .await
        .unwrap();

    let (second_user, _) = user_with_identity("test-issuer", "test-subject");
    let second_identity = Identity::new(
        identity.issuer.clone(),
        identity.subject.clone(),
        second_user.id,
    );

    let result = test
        .repository
        .create(second_user.clone(), second_identity.clone())
        .await;

    assert!(matches!(
        result,
        Err(Error::IdentityAlreadyExists(identity)) if identity == second_identity
    ));

    let user_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
            .bind(second_user.id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert!(!user_exists);
}

#[tokio::test]
async fn should_preserve_identity_uniqueness_under_concurrent_creation() {
    // Create two users concurrently with the same identity and verify that exactly one succeeds.
    let test = TestRepository::new().await;

    let (first_user, first_identity) =
        user_with_identity("concurrent-issuer", "concurrent-subject");
    let (second_user, second_identity) =
        user_with_identity("concurrent-issuer", "concurrent-subject");

    let first_result = test.repository.create(first_user.clone(), first_identity);

    let second_result = test.repository.create(second_user.clone(), second_identity);

    let (first_result, second_result) = tokio::join!(first_result, second_result);

    assert!(matches!(
        (&first_result, &second_result),
        (Ok(_), Err(Error::IdentityAlreadyExists(_)))
            | (Err(Error::IdentityAlreadyExists(_)), Ok(_))
    ));

    let persisted_identity_user_id = sqlx::query_scalar::<_, UserId>(
        "SELECT user_id FROM identities WHERE issuer = $1 AND subject = $2",
    )
    .bind("concurrent-issuer")
    .bind("concurrent-subject")
    .fetch_one(&test.pool)
    .await
    .unwrap();

    let successful_user_id = match (&first_result, &second_result) {
        (Ok(user), Err(_)) => user.id,
        (Err(_), Ok(user)) => user.id,
        _ => unreachable!("exactly one concurrent creation must succeed"),
    };

    assert_eq!(persisted_identity_user_id, successful_user_id);

    let persisted_users =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE id = $1 OR id = $2")
            .bind(first_user.id)
            .bind(second_user.id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert_eq!(persisted_users, 1);
}
