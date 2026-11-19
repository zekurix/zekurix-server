use zekurix_server::{authorization::AuthorizationRepository, router::Pagination};

use super::harness::TestHarness;

#[tokio::test]
async fn should_create_super_admin_with_all_permissions() {
    let harness = TestHarness::new().await;

    let permissions = harness.repository.list_permissions().await.unwrap();
    let roles = harness
        .repository
        .list_roles(Pagination::default())
        .await
        .unwrap();
    let super_admin = roles
        .iter()
        .find(|role| role.name().as_str() == "super-admin")
        .expect("the super-admin role should exist");

    assert_eq!(
        super_admin.permissions(),
        &permissions,
        "the super-admin role should have every permission"
    );
}
