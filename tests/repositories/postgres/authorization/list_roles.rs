use zekurix_server::{
    authorization::{AuthorizationRepository, Permission, Role, RoleName},
    router::Pagination,
};

use super::harness::TestHarness;

#[tokio::test]
async fn should_list_roles_with_permissions_ordered_by_role_id() {
    // Persist multiple roles with permissions and verify that list_roles returns
    // roles ordered by ID with each role's permissions ordered by name.
    let harness = TestHarness::new().await;

    let first_role = Role::new(
        RoleName::new("first-role").unwrap(),
        vec![Permission::RolesUpdate, Permission::RolesRead],
    );
    let second_role = Role::new(
        RoleName::new("second-role").unwrap(),
        vec![Permission::RolesDelete, Permission::RolesCreate],
    );
    let third_role = Role::new(
        RoleName::new("third-role").unwrap(),
        vec![Permission::RolesUnassign, Permission::RolesAssign],
    );

    let mut expected = harness
        .repository
        .list_roles(Pagination::default())
        .await
        .unwrap();
    expected.extend([
        Role::from_parts(
            first_role.id(),
            first_role.name().clone(),
            vec![Permission::RolesRead, Permission::RolesUpdate],
        ),
        Role::from_parts(
            second_role.id(),
            second_role.name().clone(),
            vec![Permission::RolesCreate, Permission::RolesDelete],
        ),
        Role::from_parts(
            third_role.id(),
            third_role.name().clone(),
            vec![Permission::RolesAssign, Permission::RolesUnassign],
        ),
    ]);

    harness.repository.create_role(third_role).await.unwrap();
    harness.repository.create_role(first_role).await.unwrap();
    harness.repository.create_role(second_role).await.unwrap();

    expected.sort_by_key(Role::id);

    let roles = harness
        .repository
        .list_roles(Pagination::default())
        .await
        .unwrap();

    assert_eq!(roles, expected);
}

#[tokio::test]
async fn should_list_roles_without_permissions() {
    // Persist roles without permissions and verify that list_roles returns them
    // with empty permission collections.
    let harness = TestHarness::new().await;

    let first_role = Role::new(RoleName::new("first-role").unwrap(), vec![]);
    let second_role = Role::new(RoleName::new("second-role").unwrap(), vec![]);

    let mut expected = harness
        .repository
        .list_roles(Pagination::default())
        .await
        .unwrap();
    expected.extend([
        Role::from_parts(first_role.id(), first_role.name().clone(), vec![]),
        Role::from_parts(second_role.id(), second_role.name().clone(), vec![]),
    ]);

    harness.repository.create_role(second_role).await.unwrap();
    harness.repository.create_role(first_role).await.unwrap();

    expected.sort_by_key(Role::id);

    let roles = harness
        .repository
        .list_roles(Pagination::default())
        .await
        .unwrap();

    assert_eq!(roles, expected);
}

#[tokio::test]
async fn should_apply_limit_and_offset_when_listing_roles() {
    // Persist multiple roles and verify that list_roles applies both the requested
    // limit and offset to the ordered result set.
    let harness = TestHarness::new().await;

    let mut expected = harness
        .repository
        .list_roles(Pagination::default())
        .await
        .unwrap();

    for index in 0..5 {
        let role = Role::new(
            RoleName::new(&format!("role-{index}")).unwrap(),
            vec![Permission::RolesRead],
        );

        expected.push(Role::from_parts(
            role.id(),
            role.name().clone(),
            role.permissions().to_vec(),
        ));

        harness.repository.create_role(role).await.unwrap();
    }

    expected.sort_by_key(Role::id);

    let roles = harness
        .repository
        .list_roles(Pagination {
            limit: 2,
            offset: 1,
        })
        .await
        .unwrap();

    assert_eq!(roles, expected[1..3]);
}
