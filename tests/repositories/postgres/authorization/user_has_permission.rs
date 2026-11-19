use zekurix_server::{
    authorization::{AuthorizationRepository, Permission, Role, RoleName},
    error::Error,
    user::{UserId, UserRepository},
};

use crate::repositories::postgres::fixtures::user_with_identity;

use super::harness::TestHarness;

#[tokio::test]
async fn should_return_true_when_user_has_permission() {
    // Assign a role granting the requested permission and verify that
    // user_has_permission returns true.
    let harness = TestHarness::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    harness
        .user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let role = Role::new(
        RoleName::new("test-role").unwrap(),
        vec![Permission::RolesRead],
    );
    let role_id = role.id();

    harness.repository.create_role(role).await.unwrap();

    harness
        .repository
        .assign_role_to_user(user.id, role_id)
        .await
        .unwrap();

    let has_permission = harness
        .repository
        .user_has_permission(user.id, Permission::RolesRead)
        .await
        .unwrap();

    assert!(has_permission);
}

#[tokio::test]
async fn should_return_false_when_user_does_not_have_permission() {
    // Assign a role not granting the requested permission and verify that
    // user_has_permission returns false.
    let harness = TestHarness::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    harness
        .user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let role = Role::new(
        RoleName::new("test-role").unwrap(),
        vec![Permission::RolesRead],
    );
    let role_id = role.id();

    harness.repository.create_role(role).await.unwrap();

    harness
        .repository
        .assign_role_to_user(user.id, role_id)
        .await
        .unwrap();

    let has_permission = harness
        .repository
        .user_has_permission(user.id, Permission::RolesUpdate)
        .await
        .unwrap();

    assert!(!has_permission);
}

#[tokio::test]
async fn should_return_false_when_user_has_no_roles() {
    // Persist a user without assigned roles and verify that
    // user_has_permission returns false.
    let harness = TestHarness::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    harness
        .user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let has_permission = harness
        .repository
        .user_has_permission(user.id, Permission::RolesRead)
        .await
        .unwrap();

    assert!(!has_permission);
}

#[tokio::test]
async fn should_return_true_when_permission_is_granted_by_any_assigned_role() {
    // Assign multiple roles and verify that user_has_permission returns true
    // when any assigned role grants the requested permission.
    let harness = TestHarness::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    harness
        .user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let first_role = Role::new(
        RoleName::new("first-role").unwrap(),
        vec![Permission::RolesCreate],
    );
    let first_role_id = first_role.id();

    let second_role = Role::new(
        RoleName::new("second-role").unwrap(),
        vec![Permission::RolesRead],
    );
    let second_role_id = second_role.id();

    harness.repository.create_role(first_role).await.unwrap();
    harness.repository.create_role(second_role).await.unwrap();

    harness
        .repository
        .assign_role_to_user(user.id, first_role_id)
        .await
        .unwrap();

    harness
        .repository
        .assign_role_to_user(user.id, second_role_id)
        .await
        .unwrap();

    let has_permission = harness
        .repository
        .user_has_permission(user.id, Permission::RolesRead)
        .await
        .unwrap();

    assert!(has_permission);
}

#[tokio::test]
async fn should_reject_unknown_user() {
    // Query a permission for a non-existing user ID and verify that
    // UserNotFound is returned.
    let harness = TestHarness::new().await;
    let user_id = UserId::new();

    let error = harness
        .repository
        .user_has_permission(user_id, Permission::RolesRead)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        Error::UserNotFound(error_id) if error_id == user_id
    ));
}
