use zekurix_server::{
    authorization::{AuthorizationRepository, Permission, Role, RoleId, RoleName},
    error::Error,
};

use super::common::TestRepository;

#[tokio::test]
async fn should_create_role_with_permissions() {
    // Create a role with permissions and verify that the role and all its
    // role-permission associations are correctly persisted.
    let test = TestRepository::new().await;

    let role = Role::new(
        RoleName::new("test-role").unwrap(),
        vec![Permission::RolesRead, Permission::RolesUpdate],
    );

    let role_id = role.id();

    let created = test.repository.create_role(role).await.unwrap();

    assert_eq!(created.id(), role_id);
    assert_eq!(created.name().as_str(), "test-role");
    assert_eq!(
        created.permissions(),
        &[Permission::RolesRead, Permission::RolesUpdate]
    );

    let persisted_name = sqlx::query_scalar::<_, String>("SELECT name FROM roles WHERE id = $1")
        .bind(role_id)
        .fetch_one(&test.pool)
        .await
        .unwrap();

    assert_eq!(persisted_name, "test-role");

    let persisted_permissions = sqlx::query_scalar::<_, String>(
        "SELECT permission FROM role_permissions WHERE role_id = $1 ORDER BY permission",
    )
    .bind(role_id)
    .fetch_all(&test.pool)
    .await
    .unwrap();

    assert_eq!(
        persisted_permissions,
        vec!["roles:read".to_string(), "roles:update".to_string()]
    );
}

#[tokio::test]
async fn should_create_role_without_permissions() {
    // Create a role without permissions and verify that the role is persisted
    // without any role-permission association.
    let test = TestRepository::new().await;

    let role = Role::new(RoleName::new("test-role").unwrap(), vec![]);
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let role_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM roles WHERE id = $1)")
            .bind(role_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert!(role_exists);

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
    // Create two roles with the same name and verify that the second creation
    // returns RoleAlreadyExists without leaving the second role or its permissions persisted.
    let test = TestRepository::new().await;

    let first_role = Role::new(
        RoleName::new("test-role").unwrap(),
        vec![Permission::RolesRead],
    );

    test.repository.create_role(first_role).await.unwrap();

    let second_role = Role::new(
        RoleName::new("test-role").unwrap(),
        vec![Permission::RolesUpdate],
    );

    let second_role_id = second_role.id();

    let result = test.repository.create_role(second_role).await;

    assert!(matches!(
        result,
        Err(Error::RoleAlreadyExists(name)) if name.as_str() == "test-role"
    ));

    let role_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM roles WHERE id = $1)")
            .bind(second_role_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert!(!role_exists);

    let permission_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM role_permissions WHERE role_id = $1")
            .bind(second_role_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert_eq!(permission_count, 0);
}

#[tokio::test]
async fn should_preserve_role_name_uniqueness_under_concurrent_creation() {
    // Create two roles concurrently with the same name and verify that exactly
    // one succeeds and only the successful role and its associations are persisted.
    let test = TestRepository::new().await;

    let first_role = Role::new(
        RoleName::new("concurrent-role").unwrap(),
        vec![Permission::RolesRead],
    );

    let second_role = Role::new(
        RoleName::new("concurrent-role").unwrap(),
        vec![Permission::RolesUpdate],
    );

    let first_role_id = first_role.id();
    let second_role_id = second_role.id();

    let first_result = test.repository.create_role(first_role);
    let second_result = test.repository.create_role(second_role);

    let (first_result, second_result) = tokio::join!(first_result, second_result);

    assert!(matches!(
        (&first_result, &second_result),
        (Ok(_), Err(Error::RoleAlreadyExists(_))) | (Err(Error::RoleAlreadyExists(_)), Ok(_))
    ));

    let successful_role = match (&first_result, &second_result) {
        (Ok(role), Err(_)) => role,
        (Err(_), Ok(role)) => role,
        _ => unreachable!("exactly one concurrent creation must succeed"),
    };

    let persisted_role_id = sqlx::query_scalar::<_, RoleId>("SELECT id FROM roles WHERE name = $1")
        .bind("concurrent-role")
        .fetch_one(&test.pool)
        .await
        .unwrap();

    assert_eq!(persisted_role_id, successful_role.id());

    let persisted_permissions = sqlx::query_scalar::<_, String>(
        "SELECT permission FROM role_permissions WHERE role_id = $1 ORDER BY permission",
    )
    .bind(successful_role.id())
    .fetch_all(&test.pool)
    .await
    .unwrap();

    let expected_permissions: Vec<_> = successful_role
        .permissions()
        .iter()
        .map(|permission| permission.as_str().to_string())
        .collect();

    assert_eq!(persisted_permissions, expected_permissions);

    let persisted_roles =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM roles WHERE id = $1 OR id = $2")
            .bind(first_role_id)
            .bind(second_role_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert_eq!(persisted_roles, 1);
}
