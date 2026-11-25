use zekurix_server::{
    authorization::{AuthorizationRepository, Permission, Role, RoleFilter, RoleName},
    router::PaginationQuery,
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
        .list_roles(&RoleFilter::default(), PaginationQuery::default())
        .await
        .unwrap()
        .items;
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
        .list_roles(&RoleFilter::default(), PaginationQuery::default())
        .await
        .unwrap()
        .items;

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
        .list_roles(&RoleFilter::default(), PaginationQuery::default())
        .await
        .unwrap()
        .items;
    expected.extend([
        Role::from_parts(first_role.id(), first_role.name().clone(), vec![]),
        Role::from_parts(second_role.id(), second_role.name().clone(), vec![]),
    ]);

    harness.repository.create_role(second_role).await.unwrap();
    harness.repository.create_role(first_role).await.unwrap();

    expected.sort_by_key(Role::id);

    let roles = harness
        .repository
        .list_roles(&RoleFilter::default(), PaginationQuery::default())
        .await
        .unwrap()
        .items;

    assert_eq!(roles, expected);
}

#[tokio::test]
async fn should_apply_limit_and_offset_when_listing_roles() {
    // Persist multiple roles and verify that list_roles applies both the requested
    // limit and offset to the ordered result set.
    let harness = TestHarness::new().await;

    let mut expected = harness
        .repository
        .list_roles(&RoleFilter::default(), PaginationQuery::default())
        .await
        .unwrap()
        .items;

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
        .list_roles(
            &RoleFilter::default(),
            PaginationQuery {
                limit: 2,
                offset: 1,
            },
        )
        .await
        .unwrap()
        .items;

    assert_eq!(roles, expected[1..3]);
}

#[tokio::test]
async fn should_filter_roles_by_exact_name() {
    let harness = TestHarness::new().await;

    let matching_role = Role::new(
        RoleName::new("matching-role").unwrap(),
        vec![Permission::RolesRead],
    );
    let other_role = Role::new(
        RoleName::new("other-role").unwrap(),
        vec![Permission::RolesDelete],
    );

    let expected = Role::from_parts(
        matching_role.id(),
        matching_role.name().clone(),
        matching_role.permissions().to_vec(),
    );

    harness.repository.create_role(matching_role).await.unwrap();
    harness.repository.create_role(other_role).await.unwrap();

    let filter = RoleFilter {
        name: Some(RoleName::new("matching-role").unwrap()),
    };

    let roles = harness
        .repository
        .list_roles(&filter, PaginationQuery::default())
        .await
        .unwrap()
        .items;

    assert_eq!(roles, vec![expected]);
}

#[tokio::test]
async fn should_return_empty_list_when_no_role_matches_name() {
    let harness = TestHarness::new().await;

    let role = Role::new(
        RoleName::new("existing-role").unwrap(),
        vec![Permission::RolesRead],
    );

    harness.repository.create_role(role).await.unwrap();

    let filter = RoleFilter {
        name: Some(RoleName::new("unknown-role").unwrap()),
    };

    let roles = harness
        .repository
        .list_roles(&filter, PaginationQuery::default())
        .await
        .unwrap()
        .items;

    assert!(roles.is_empty());
}
