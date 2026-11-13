use zekurix_server::{
    authorization::{AuthorizationRepository, Role, RoleId, RoleName},
    error::Error,
    user::{UserId, UserRepository},
};

use super::common::TestRepository;

use crate::repositories::postgres::common::user_with_identity;

#[tokio::test]
async fn should_unassign_role_from_user() {
    // Assign a role to a user, unassign it, and verify that the user-role
    // association is removed.
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
        .unassign_role_from_user(user.id, role_id)
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

    assert_eq!(association_count, 0);
}

#[tokio::test]
async fn should_be_idempotent_when_role_is_not_assigned() {
    // Unassign a role that is not assigned to the existing user and verify that
    // the operation succeeds and no association exists.
    let test = TestRepository::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let role_id = RoleId::new();

    test.repository
        .unassign_role_from_user(user.id, role_id)
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

    assert_eq!(association_count, 0);
}

#[tokio::test]
async fn should_reject_unknown_user() {
    // Unassign a role from a non-existing user and verify that UserNotFound is returned.
    let test = TestRepository::new().await;

    let user_id = UserId::new();
    let role_id = RoleId::new();

    let error = test
        .repository
        .unassign_role_from_user(user_id, role_id)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        Error::UserNotFound(error_id) if error_id == user_id
    ));
}
