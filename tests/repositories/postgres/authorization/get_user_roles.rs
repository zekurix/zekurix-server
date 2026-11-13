use zekurix_server::{
    authorization::{AuthorizationRepository, Role, RoleName},
    error::Error,
    user::{UserId, UserRepository},
};

use super::common::TestRepository;
use crate::repositories::postgres::common::user_with_identity;

#[tokio::test]
async fn should_get_user_roles_ordered_by_name() {
    // Assign multiple roles to a user and verify that get_user_roles returns
    // the assigned roles ordered by role name.
    let test = TestRepository::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let first_role = Role::new(RoleName::new("alpha-role").unwrap(), vec![]);
    let second_role = Role::new(RoleName::new("beta-role").unwrap(), vec![]);
    let third_role = Role::new(RoleName::new("gamma-role").unwrap(), vec![]);

    let expected = vec![
        Role::from_parts(first_role.id(), first_role.name().clone(), vec![]),
        Role::from_parts(second_role.id(), second_role.name().clone(), vec![]),
        Role::from_parts(third_role.id(), third_role.name().clone(), vec![]),
    ];

    let first_role_id = first_role.id();
    let second_role_id = second_role.id();
    let third_role_id = third_role.id();

    test.repository.create_role(third_role).await.unwrap();
    test.repository.create_role(first_role).await.unwrap();
    test.repository.create_role(second_role).await.unwrap();

    test.repository
        .assign_role_to_user(user.id, third_role_id)
        .await
        .unwrap();

    test.repository
        .assign_role_to_user(user.id, first_role_id)
        .await
        .unwrap();

    test.repository
        .assign_role_to_user(user.id, second_role_id)
        .await
        .unwrap();

    let roles = test.repository.get_user_roles(user.id).await.unwrap();

    assert_eq!(roles, expected);
}

#[tokio::test]
async fn should_return_empty_roles_for_user_without_roles() {
    // Persist a user without any assigned role and verify that get_user_roles
    // returns an empty collection rather than treating the user as missing.
    let test = TestRepository::new().await;

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    let roles = test.repository.get_user_roles(user.id).await.unwrap();

    assert!(roles.is_empty());
}

#[tokio::test]
async fn should_reject_unknown_user() {
    // Query roles for a non-existing user ID and verify that UserNotFound is returned.
    let test = TestRepository::new().await;
    let user_id = UserId::new();

    let error = test.repository.get_user_roles(user_id).await.unwrap_err();

    assert!(matches!(
        error,
        Error::UserNotFound(error_id) if error_id == user_id
    ));
}
