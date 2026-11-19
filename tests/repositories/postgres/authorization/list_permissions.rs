use zekurix_server::authorization::{AuthorizationRepository, Permission};
use zekurix_server::error::Error;

use super::harness::TestHarness;

#[tokio::test]
async fn should_list_permissions() {
    // List permissions and verify that all persisted permissions are returned
    // ordered by name.
    let harness = TestHarness::new().await;

    let permissions = harness.repository.list_permissions().await.unwrap();

    assert_eq!(
        permissions,
        vec![
            Permission::RolesAssign,
            Permission::RolesCreate,
            Permission::RolesDelete,
            Permission::RolesRead,
            Permission::RolesUnassign,
            Permission::RolesUpdate,
            Permission::UsersCreate,
            Permission::UsersDelete,
            Permission::UsersRead,
        ]
    );
}

#[tokio::test]
async fn should_reject_invalid_persisted_permission() {
    // Persist an unknown permission value and verify that listing permissions
    // returns an internal error instead of exposing invalid domain data.
    let harness = TestHarness::new().await;

    sqlx::query("INSERT INTO permissions (name) VALUES ($1)")
        .bind("unknown:permission")
        .execute(&harness.pool)
        .await
        .unwrap();

    let result = harness.repository.list_permissions().await;

    assert!(matches!(result, Err(Error::InternalError(_))));
}
