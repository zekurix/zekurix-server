use zekurix_server::{
    authorization::{AuthorizationRepository, Permission, Role, RoleId, RoleName},
    error::Error,
};

use super::common::TestRepository;

#[tokio::test]
async fn should_get_role_with_permissions_ordered_by_name() {
    // Persist a role with multiple permissions and verify that get_role returns
    // the expected role with its permissions ordered by permission name.
    let test = TestRepository::new().await;

    let role = Role::new(
        RoleName::new("test-role").unwrap(),
        vec![
            Permission::RolesUpdate,
            Permission::RolesCreate,
            Permission::RolesRead,
        ],
    );
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let found = test.repository.get_role(role_id).await.unwrap();

    assert_eq!(found.id(), role_id);
    assert_eq!(found.name().as_str(), "test-role");
    assert_eq!(
        found.permissions(),
        &[
            Permission::RolesCreate,
            Permission::RolesRead,
            Permission::RolesUpdate,
        ]
    );
}

#[tokio::test]
async fn should_get_role_without_permissions() {
    // Persist a role without permissions and verify that get_role returns it
    // with an empty permissions collection.
    let test = TestRepository::new().await;

    let role = Role::new(RoleName::new("test-role").unwrap(), vec![]);
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let found = test.repository.get_role(role_id).await.unwrap();

    assert_eq!(found.id(), role_id);
    assert_eq!(found.name().as_str(), "test-role");
    assert!(found.permissions().is_empty());
}

#[tokio::test]
async fn should_reject_unknown_role() {
    // Look up a non-existing role ID and verify that RoleNotFound is returned.
    let test = TestRepository::new().await;
    let id = RoleId::new();

    let error = test.repository.get_role(id).await.unwrap_err();

    assert!(matches!(
        error,
        Error::RoleNotFound(error_id) if error_id == id
    ));
}
