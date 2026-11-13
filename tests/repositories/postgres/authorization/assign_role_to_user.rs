use zekurix_server::{
    authorization::{AuthorizationRepository, Role, RoleId, RoleName},
    error::Error,
    user::{UserId, UserRepository},
};

use super::common::TestRepository;
use crate::repositories::postgres::common::user_with_identity;

#[tokio::test]
async fn should_assign_role_to_user() {
    // Persist a user and a role, assign the role to the user, and verify that
    // exactly one user-role association is persisted.
    let test = TestRepository::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let role = Role::new(RoleName::new("test-role").unwrap(), vec![]);
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    test.repository
        .assign_role_to_user(user.id, role_id)
        .await
        .unwrap();

    let association_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = $1 AND role_id = $2",
    )
    .bind(user.id)
    .bind(role_id)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert_eq!(association_count, 1);
}

#[tokio::test]
async fn should_be_idempotent_when_role_is_already_assigned() {
    // Assign the same role to the same user twice and verify that both operations
    // succeed while only one user-role association exists.
    let test = TestRepository::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let role = Role::new(RoleName::new("test-role").unwrap(), vec![]);
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    test.repository
        .assign_role_to_user(user.id, role_id)
        .await
        .unwrap();

    test.repository
        .assign_role_to_user(user.id, role_id)
        .await
        .unwrap();

    let association_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = $1 AND role_id = $2",
    )
    .bind(user.id)
    .bind(role_id)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert_eq!(association_count, 1);
}

#[tokio::test]
async fn should_reject_unknown_user() {
    // Assign an existing role to a non-existing user and verify that UserNotFound
    // is returned without creating an association.
    let test = TestRepository::new().await;

    let user_id = UserId::new();

    let role = Role::new(RoleName::new("test-role").unwrap(), vec![]);
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let error = test
        .repository
        .assign_role_to_user(user_id, role_id)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        Error::UserNotFound(error_id) if error_id == user_id
    ));

    let association_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = $1 AND role_id = $2",
    )
    .bind(user_id)
    .bind(role_id)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert_eq!(association_count, 0);
}

#[tokio::test]
async fn should_reject_unknown_role() {
    // Assign a non-existing role to an existing user and verify that RoleNotFound
    // is returned without creating an association.
    let test = TestRepository::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let role_id = RoleId::new();

    let error = test
        .repository
        .assign_role_to_user(user.id, role_id)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        Error::RoleNotFound(error_id) if error_id == role_id
    ));

    let association_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = $1 AND role_id = $2",
    )
    .bind(user.id)
    .bind(role_id)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert_eq!(association_count, 0);
}

#[tokio::test]
async fn should_preserve_assignment_uniqueness_under_concurrent_assignment() {
    // Assign the same role concurrently to the same user and verify that both
    // operations succeed while exactly one user-role association is persisted.
    let test = TestRepository::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let role = Role::new(RoleName::new("test-role").unwrap(), vec![]);
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let first_assignment = test.repository.assign_role_to_user(user.id, role_id);
    let second_assignment = test.repository.assign_role_to_user(user.id, role_id);

    let (first_result, second_result) = tokio::join!(first_assignment, second_assignment);

    assert!(first_result.is_ok());
    assert!(second_result.is_ok());

    let association_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_roles WHERE user_id = $1 AND role_id = $2",
    )
    .bind(user.id)
    .bind(role_id)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert_eq!(association_count, 1);
}
