use zekurix_server::{
    authorization::{AuthorizationRepository, Permission, Role, RoleName},
    error::Error,
    user::{UserId, UserRepository},
};

use super::common::TestRepository;
use crate::repositories::postgres::common::user_with_identity;

#[tokio::test]
async fn should_get_distinct_user_permissions_ordered_by_name() {
    // Assign multiple roles with overlapping permissions to a user and verify
    // that get_user_permissions returns distinct permissions ordered by name.
    let test = TestRepository::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let first_role = Role::new(
        RoleName::new("first-role").unwrap(),
        vec![Permission::RolesUpdate, Permission::RolesRead],
    );
    let first_role_id = first_role.id();

    let second_role = Role::new(
        RoleName::new("second-role").unwrap(),
        vec![Permission::RolesCreate, Permission::RolesRead],
    );
    let second_role_id = second_role.id();

    test.repository.create_role(first_role).await.unwrap();
    test.repository.create_role(second_role).await.unwrap();

    test.repository
        .assign_role_to_user(user.id, first_role_id)
        .await
        .unwrap();

    test.repository
        .assign_role_to_user(user.id, second_role_id)
        .await
        .unwrap();

    let permissions = test.repository.get_user_permissions(user.id).await.unwrap();

    assert_eq!(
        permissions,
        vec![
            Permission::RolesCreate,
            Permission::RolesRead,
            Permission::RolesUpdate,
        ]
    );
}

#[tokio::test]
async fn should_return_empty_permissions_for_user_without_roles() {
    // Persist a user without assigned roles and verify that get_user_permissions
    // returns an empty collection rather than treating the user as missing.
    let test = TestRepository::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let permissions = test.repository.get_user_permissions(user.id).await.unwrap();

    assert!(permissions.is_empty());
}

#[tokio::test]
async fn should_return_empty_permissions_for_roles_without_permissions() {
    // Assign a role with no permissions to a user and verify that
    // get_user_permissions returns an empty collection.
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

    let permissions = test.repository.get_user_permissions(user.id).await.unwrap();

    assert!(permissions.is_empty());
}

#[tokio::test]
async fn should_reject_unknown_user() {
    // Query permissions for a non-existing user ID and verify that UserNotFound is returned.
    let test = TestRepository::new().await;
    let user_id = UserId::new();

    let error = test
        .repository
        .get_user_permissions(user_id)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        Error::UserNotFound(error_id) if error_id == user_id
    ));
}
