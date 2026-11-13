use zekurix_server::{
    authorization::{AuthorizationRepository, Permission, Role, RoleId, RoleName},
    error::Error,
    user::UserRepository,
};

use super::common::TestRepository;

use crate::repositories::postgres::common::user_with_identity;

#[tokio::test]
async fn should_delete_role_and_cascade_associations() {
    // Delete an existing role and verify that the role, its role-permission
    // associations, and its user-role associations are removed by database cascades.
    let test = TestRepository::new().await;

    let role = Role::new(
        RoleName::new("test-role").unwrap(),
        vec![Permission::RolesRead],
    );
    let role_id = role.id();

    test.repository.create_role(role).await.unwrap();

    let (user, identity) = user_with_identity("test-issuer", "test-subject");

    test.user_repository
        .create(user.clone(), identity)
        .await
        .unwrap();

    test.repository
        .assign_role_to_user(user.id, role_id)
        .await
        .unwrap();

    test.repository.delete_role(role_id).await.unwrap();

    let role_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM roles WHERE id = $1)")
            .bind(role_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert!(!role_exists);

    let role_permission_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert_eq!(role_permission_count, 0);

    let user_role_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_roles WHERE role_id = $1")
            .bind(role_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert_eq!(user_role_count, 0);
}

#[tokio::test]
async fn should_reject_unknown_role() {
    // Delete a non-existing role ID and verify that RoleNotFound is returned.
    let test = TestRepository::new().await;
    let id = RoleId::new();

    let error = test.repository.delete_role(id).await.unwrap_err();

    assert!(matches!(error, Error::RoleNotFound(error_id) if error_id == id));
}
