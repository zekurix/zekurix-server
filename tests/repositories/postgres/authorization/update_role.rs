use zekurix_server::{
    authorization::{AuthorizationRepository, Permission, Role, RoleId, RoleName},
    error::Error,
};

use super::common::TestRepository;

#[tokio::test]
async fn should_update_role_name_and_permissions() {
    // Persist a role, update its name and permissions, and verify that both the
    // role row and its role-permission associations reflect the new state.
    let test = TestRepository::new().await;

    let role = Role::new(
        RoleName::new("original-role").unwrap(),
        vec![Permission::RolesRead],
    );
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let updated_role = Role::from_parts(
        role_id,
        RoleName::new("updated-role").unwrap(),
        vec![Permission::RolesUpdate, Permission::RolesDelete],
    );

    let updated = test.repository.update_role(updated_role).await.unwrap();

    assert_eq!(
        updated,
        Role::from_parts(
            role_id,
            RoleName::new("updated-role").unwrap(),
            vec![Permission::RolesUpdate, Permission::RolesDelete],
        )
    );

    let persisted = test.repository.get_role(role_id).await.unwrap();

    assert_eq!(
        persisted,
        Role::from_parts(
            role_id,
            RoleName::new("updated-role").unwrap(),
            vec![Permission::RolesDelete, Permission::RolesUpdate],
        )
    );
}

#[tokio::test]
async fn should_remove_all_role_permissions() {
    // Update a role that has permissions with an empty permission list and verify
    // that all previous role-permission associations are removed.
    let test = TestRepository::new().await;

    let role = Role::new(
        RoleName::new("test-role").unwrap(),
        vec![Permission::RolesRead, Permission::RolesUpdate],
    );
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let updated_role = Role::from_parts(role_id, RoleName::new("test-role").unwrap(), vec![]);

    test.repository.update_role(updated_role).await.unwrap();

    let persisted = test.repository.get_role(role_id).await.unwrap();

    assert_eq!(
        persisted,
        Role::from_parts(role_id, RoleName::new("test-role").unwrap(), vec![],)
    );

    let permission_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert_eq!(permission_count, 0);
}

#[tokio::test]
async fn should_reject_existing_role_name_and_rollback() {
    // Update a role to an already-used name and verify that RoleAlreadyExists is
    // returned while the role's original name and permissions remain unchanged.
    let test = TestRepository::new().await;

    let existing_role = Role::new(RoleName::new("existing-role").unwrap(), vec![]);

    test.repository.create_role(existing_role).await.unwrap();

    let role = Role::new(
        RoleName::new("original-role").unwrap(),
        vec![Permission::RolesRead],
    );
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let updated_role = Role::from_parts(
        role_id,
        RoleName::new("existing-role").unwrap(),
        vec![Permission::RolesUpdate],
    );

    let error = test.repository.update_role(updated_role).await.unwrap_err();

    assert!(matches!(
        error,
        Error::RoleAlreadyExists(name) if name == RoleName::new("existing-role").unwrap()
    ));

    let persisted = test.repository.get_role(role_id).await.unwrap();

    assert_eq!(
        persisted,
        Role::from_parts(
            role_id,
            RoleName::new("original-role").unwrap(),
            vec![Permission::RolesRead],
        )
    );
}

#[tokio::test]
async fn should_reject_unknown_role() {
    // Update a non-existing role ID and verify that RoleNotFound is returned
    // without persisting any role-permission association.
    let test = TestRepository::new().await;

    let role_id = RoleId::new();
    let role = Role::from_parts(
        role_id,
        RoleName::new("unknown-role").unwrap(),
        vec![Permission::RolesRead],
    );

    let error = test.repository.update_role(role).await.unwrap_err();

    assert!(matches!(
        error,
        Error::RoleNotFound(error_id) if error_id == role_id
    ));

    let permission_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert_eq!(permission_count, 0);
}

#[tokio::test]
async fn should_preserve_role_consistency_under_concurrent_updates()
    // Update the same role concurrently with two different names and permission
    // sets, and verify that the final persisted role exactly matches one complete
    // update without mixing data from both transactions.
    let test = TestRepository::new().await;

    let role = Role::new(
        RoleName::new("original-role").unwrap(),
        vec![Permission::RolesRead],
    );
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let first_update = Role::from_parts(
        role_id,
        RoleName::new("first-update").unwrap(),
        vec![Permission::RolesCreate, Permission::RolesRead],
    );

    let second_update = Role::from_parts(
        role_id,
        RoleName::new("second-update").unwrap(),
        vec![Permission::RolesDelete, Permission::RolesUpdate],
    );

    let first_result = test.repository.update_role(first_update);
    let second_result = test.repository.update_role(second_update);

    let (first_result, second_result) = tokio::join!(first_result, second_result);

    assert!(first_result.is_ok());
    assert!(second_result.is_ok());

    let persisted = test.repository.get_role(role_id).await.unwrap();

    let expected_first = Role::from_parts(
        role_id,
        RoleName::new("first-update").unwrap(),
        vec![Permission::RolesCreate, Permission::RolesRead],
    );

    let expected_second = Role::from_parts(
        role_id,
        RoleName::new("second-update").unwrap(),
        vec![Permission::RolesDelete, Permission::RolesUpdate],
    );

    assert!(
        persisted == expected_first || persisted == expected_second,
        "the persisted role must exactly match one complete concurrent update"
    );
}
