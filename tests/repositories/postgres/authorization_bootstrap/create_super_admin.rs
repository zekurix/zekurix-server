use zekurix_server::{
    authorization::{AuthorizationBootstrapRepository, AuthorizationRepository, RoleName},
    error::Error,
    router::Pagination,
    user::{UserId, UserRepository},
};

use crate::repositories::postgres::common::user_with_identity;

use super::common::TestRepository;

const SUPER_ADMIN_ROLE_NAME: &str = "super-admin";
const ISSUER: &str = "https://auth.example.com/my-issuer";
const SUBJECT: &str = "admin";

#[tokio::test]
async fn should_create_user_and_identity_for_super_admin() {
    // When the configured identity does not exist, the bootstrap must create
    // both the user and its identity.
    let test = TestRepository::new().await;
    let (user, identity) = user_with_identity(ISSUER, SUBJECT);
    let user_id = user.id;

    let mut tx = test.repository.begin().await.unwrap();
    tx.create_super_admin(
        user,
        identity,
        RoleName::new(SUPER_ADMIN_ROLE_NAME).unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let persisted_user_id = sqlx::query_scalar::<_, UserId>(
        r#"
        SELECT u.id
        FROM users AS u
        INNER JOIN identities AS i ON i.user_id = u.id
        WHERE i.issuer = $1 AND i.subject = $2
        "#,
    )
    .bind(ISSUER)
    .bind(SUBJECT)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert_eq!(persisted_user_id, user_id);
}

#[tokio::test]
async fn should_assign_super_admin_role_to_user() {
    // The created super-admin user must be assigned the requested role.
    let test = TestRepository::new().await;
    let (user, identity) = user_with_identity(ISSUER, SUBJECT);
    let user_id = user.id;

    let mut tx = test.repository.begin().await.unwrap();
    tx.create_super_admin(
        user,
        identity,
        RoleName::new(SUPER_ADMIN_ROLE_NAME).unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let role_assigned = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM user_roles AS ur
            INNER JOIN roles AS r ON r.id = ur.role_id
            WHERE ur.user_id = $1 AND r.name = $2
        )
        "#,
    )
    .bind(user_id)
    .bind(SUPER_ADMIN_ROLE_NAME)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert!(role_assigned);
}

#[tokio::test]
async fn should_reuse_existing_user_when_identity_already_exists() {
    // When the configured identity already exists, the bootstrap must reuse
    // its existing user and must not persist the newly supplied user.
    let test = TestRepository::new().await;

    let (existing_user, existing_identity) = user_with_identity(ISSUER, SUBJECT);
    let existing_user_id = existing_user.id;

    test.user_repository
        .create(existing_user, existing_identity)
        .await
        .unwrap();

    let (new_user, new_identity) = user_with_identity(ISSUER, SUBJECT);
    let new_user_id = new_user.id;

    let mut tx = test.repository.begin().await.unwrap();
    tx.create_super_admin(
        new_user,
        new_identity,
        RoleName::new(SUPER_ADMIN_ROLE_NAME).unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let assigned_user_id = sqlx::query_scalar::<_, UserId>(
        r#"
        SELECT user_id
        FROM identities
        WHERE issuer = $1 AND subject = $2
        "#,
    )
    .bind(ISSUER)
    .bind(SUBJECT)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert_eq!(assigned_user_id, existing_user_id);

    let new_user_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
            .bind(new_user_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert!(!new_user_exists);
}

#[tokio::test]
async fn should_succeed_when_super_admin_role_is_already_assigned() {
    // Assigning the role when the existing user already has it must succeed
    // without creating a duplicate assignment.
    let test = TestRepository::new().await;

    let (existing_user, existing_identity) = user_with_identity(ISSUER, SUBJECT);
    let existing_user_id = existing_user.id;

    test.user_repository
        .create(existing_user, existing_identity)
        .await
        .unwrap();
    let role = test
        .authorization_repository
        .list_roles(Pagination::default())
        .await
        .unwrap()
        .into_iter()
        .find(|role| role.name().as_str() == SUPER_ADMIN_ROLE_NAME)
        .unwrap();

    test.authorization_repository
        .assign_role_to_user(existing_user_id, role.id())
        .await
        .unwrap();

    let (new_user, new_identity) = user_with_identity(ISSUER, SUBJECT);

    let mut tx = test.repository.begin().await.unwrap();
    tx.create_super_admin(
        new_user,
        new_identity,
        RoleName::new(SUPER_ADMIN_ROLE_NAME).unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let assignment_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM user_roles AS ur
        INNER JOIN roles AS r ON r.id = ur.role_id
        WHERE ur.user_id = $1 AND r.name = $2
        "#,
    )
    .bind(existing_user_id)
    .bind(SUPER_ADMIN_ROLE_NAME)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert_eq!(assignment_count, 1);
}

#[tokio::test]
async fn should_fail_when_super_admin_role_does_not_exist() {
    // The bootstrap must fail when the requested super-admin role does not exist.
    let test = TestRepository::new().await;
    let (user, identity) = user_with_identity(ISSUER, SUBJECT);

    let mut tx = test.repository.begin().await.unwrap();

    let result = tx
        .create_super_admin(user, identity, RoleName::new("missing-role").unwrap())
        .await;

    assert!(matches!(result, Err(Error::InternalError(_))));
}

#[tokio::test]
async fn should_rollback_user_and_identity_when_role_assignment_fails() {
    // Failure to assign the super-admin role must roll back the user and identity
    // created earlier in the same transaction.
    let test = TestRepository::new().await;
    let (user, identity) = user_with_identity(ISSUER, SUBJECT);
    let user_id = user.id;

    let mut tx = test.repository.begin().await.unwrap();

    let result = tx
        .create_super_admin(user, identity, RoleName::new("missing-role").unwrap())
        .await;

    assert!(matches!(result, Err(Error::InternalError(_))));

    drop(tx);

    let user_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
            .bind(user_id)
            .fetch_one(&test.pool)
            .await
            .unwrap();

    assert!(!user_exists);

    let identity_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM identities
            WHERE issuer = $1 AND subject = $2
        )
        "#,
    )
    .bind(ISSUER)
    .bind(SUBJECT)
    .fetch_one(&test.pool)
    .await
    .unwrap();

    assert!(!identity_exists);
}
