use std::sync::Arc;

use crate::error::{Error, Result};
use crate::router::{Page, Pagination};
use crate::user::UserId;

use super::role::{Role, RoleFilter, RoleId, RoleName, RoleSummary};
use super::{AuthorizationRepository, Permission};

#[derive(Clone)]
pub struct AuthorizationService {
    repository: Arc<dyn AuthorizationRepository>,
}

impl AuthorizationService {
    pub fn new(repository: Arc<dyn AuthorizationRepository>) -> Result<Self> {
        Ok(Self { repository })
    }

    pub async fn list_permissions(&self) -> Result<Vec<Permission>> {
        self.repository.list_permissions().await
    }

    pub async fn create_role(
        &self,
        actor_id: UserId,
        name: RoleName,
        permissions: Vec<Permission>,
    ) -> Result<Role> {
        let role = Role::try_new(name, permissions)?;
        self.ensure_user_has_permissions(actor_id, role.permissions())
            .await?;
        self.repository.create_role(role).await
    }

    pub async fn get_role(&self, id: RoleId) -> Result<Role> {
        self.repository.get_role(id).await
    }

    pub async fn list_roles(
        &self,
        filter: &RoleFilter,
        pagination: Pagination,
    ) -> Result<Page<Role>> {
        self.repository.list_roles(filter, pagination).await
    }

    pub async fn update_role(
        &self,
        actor_id: UserId,
        id: RoleId,
        name: RoleName,
        permissions: Vec<Permission>,
    ) -> Result<Role> {
        let role = Role::try_from_parts(id, name, permissions)?;
        self.ensure_user_has_permissions(actor_id, role.permissions())
            .await?;
        self.repository.update_role(role).await
    }

    pub async fn delete_role(&self, id: RoleId) -> Result<()> {
        self.repository.delete_role(id).await
    }

    pub async fn list_user_roles(&self, user_id: UserId) -> Result<Vec<RoleSummary>> {
        self.repository.list_user_roles(user_id).await
    }

    pub async fn assign_role_to_user(
        &self,
        actor_id: UserId,
        target_user_id: UserId,
        role_id: RoleId,
    ) -> Result<()> {
        let role = self.repository.get_role(role_id).await?;
        self.ensure_user_has_permissions(actor_id, role.permissions())
            .await?;
        self.repository
            .assign_role_to_user(target_user_id, role_id)
            .await
    }

    pub async fn unassign_role_from_user(&self, user_id: UserId, role_id: RoleId) -> Result<()> {
        self.repository
            .unassign_role_from_user(user_id, role_id)
            .await
    }

    pub async fn list_user_permissions(&self, user_id: UserId) -> Result<Vec<Permission>> {
        self.repository.list_user_permissions(user_id).await
    }

    pub async fn user_has_permission(
        &self,
        user_id: UserId,
        permission: Permission,
    ) -> Result<bool> {
        self.repository
            .user_has_permission(user_id, permission)
            .await
    }

    async fn ensure_user_has_permissions(
        &self,
        actor_id: UserId,
        permissions: &[Permission],
    ) -> Result<()> {
        let user_permissions = self.repository.list_user_permissions(actor_id).await?;
        if permissions
            .iter()
            .all(|permission| user_permissions.contains(permission))
        {
            Ok(())
        } else {
            Err(Error::Forbidden)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::authorization::MockAuthorizationRepository;
    use crate::error::Error;

    use super::*;

    fn service(repository: MockAuthorizationRepository) -> AuthorizationService {
        AuthorizationService::new(Arc::new(repository)).unwrap()
    }

    #[tokio::test]
    async fn should_list_permissions() {
        let expected = vec![
            Permission::RolesCreate,
            Permission::RolesRead,
            Permission::RolesUpdate,
            Permission::RolesDelete,
            Permission::RolesAssign,
            Permission::RolesUnassign,
        ];

        let mut repository = MockAuthorizationRepository::new();
        repository.expect_list_permissions().times(1).return_once({
            let expected = expected.clone();
            move || Ok(expected)
        });

        let permissions = service(repository).list_permissions().await.unwrap();
        assert_eq!(permissions, expected);
    }

    #[tokio::test]
    async fn should_propagate_list_permissions_error() {
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_permissions()
            .times(1)
            .return_once(|| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository).list_permissions().await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_create_role() {
        let actor_id = UserId::new();
        let name = RoleName::new("admin").unwrap();
        let permissions = vec![Permission::RolesRead, Permission::RolesUpdate];
        let expected_name = name.clone();
        let expected_permissions = permissions.clone();
        let expected_name_for_match = expected_name.clone();
        let expected_permissions_for_match = expected_permissions.clone();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == actor_id)
            .times(1)
            .return_once({
                let expected_permissions = expected_permissions.clone();
                move |_| Ok(expected_permissions)
            });
        repository
            .expect_create_role()
            .withf(move |role| {
                role.name() == &expected_name_for_match
                    && role.permissions() == expected_permissions_for_match.as_slice()
            })
            .times(1)
            .return_once(Ok);

        let role = service(repository)
            .create_role(actor_id, name, permissions)
            .await
            .unwrap();

        assert_eq!(role.name(), &expected_name);
        assert_eq!(role.permissions(), expected_permissions.as_slice());
    }

    #[tokio::test]
    async fn should_reject_duplicate_permissions_when_creating_role() {
        let actor_id = UserId::new();
        let name = RoleName::new("admin").unwrap();
        let permissions = vec![Permission::RolesRead, Permission::RolesRead];

        let mut repository = MockAuthorizationRepository::new();
        repository.expect_create_role().times(0);

        let result = service(repository)
            .create_role(actor_id, name, permissions)
            .await;

        assert!(matches!(
            result,
            Err(Error::DuplicatePermission(Permission::RolesRead))
        ));
    }

    #[tokio::test]
    async fn should_reject_permissions_the_user_does_not_have_when_creating_role() {
        let actor_id = UserId::new();
        let name = RoleName::new("admin").unwrap();
        let permissions = vec![Permission::RolesRead, Permission::RolesDelete];

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == actor_id)
            .times(1)
            .return_once(|_| Ok(vec![Permission::RolesRead]));
        repository.expect_create_role().times(0);

        let result = service(repository)
            .create_role(actor_id, name, permissions)
            .await;

        assert!(matches!(result, Err(Error::Forbidden)));
    }

    #[tokio::test]
    async fn should_propagate_create_role_error() {
        let actor_id = UserId::new();
        let name = RoleName::new("admin").unwrap();
        let permissions = vec![Permission::RolesRead];

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == actor_id)
            .times(1)
            .return_once(|_| Ok(vec![Permission::RolesRead]));
        repository
            .expect_create_role()
            .times(1)
            .return_once(|_| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository)
            .create_role(actor_id, name, permissions)
            .await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_get_role() {
        let id = RoleId::new();
        let expected = Role::from_parts(
            id,
            RoleName::new("admin").unwrap(),
            vec![Permission::RolesRead],
        );
        let expected_name = expected.name().clone();
        let expected_permissions = expected.permissions().to_vec();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_get_role()
            .withf(move |actual_id| *actual_id == id)
            .times(1)
            .return_once(|_| Ok(expected));

        let role = service(repository).get_role(id).await.unwrap();

        assert_eq!(role.id(), id);
        assert_eq!(role.name(), &expected_name);
        assert_eq!(role.permissions(), expected_permissions.as_slice());
    }

    #[tokio::test]
    async fn should_propagate_get_role_error() {
        let id = RoleId::new();
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_get_role()
            .withf(move |actual_id| *actual_id == id)
            .times(1)
            .return_once(|_| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository).get_role(id).await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_list_roles() {
        let filter = RoleFilter::default();
        let pagination = Pagination::default();
        let expected = vec![Role::from_parts(
            RoleId::new(),
            RoleName::new("admin").unwrap(),
            vec![Permission::RolesRead],
        )];
        let expected_id = expected[0].id();
        let expected_name = expected[0].name().clone();
        let expected_permissions = expected[0].permissions().to_vec();
        let expected_filter = filter.clone();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_roles()
            .withf(move |actual_filter, actual_pagination| {
                *actual_filter == expected_filter && *actual_pagination == pagination
            })
            .times(1)
            .return_once(move |_, _| Ok(Page::new(expected, pagination, 1)));

        let roles = service(repository)
            .list_roles(&filter, pagination)
            .await
            .unwrap();

        assert_eq!(roles.items.len(), 1);
        assert_eq!(roles.items[0].id(), expected_id);
        assert_eq!(roles.items[0].name(), &expected_name);
        assert_eq!(
            roles.items[0].permissions(),
            expected_permissions.as_slice()
        );
        assert_eq!(roles.pagination, pagination);
        assert_eq!(roles.total, 1);
    }

    #[tokio::test]
    async fn should_propagate_list_roles_error() {
        let filter = RoleFilter::default();
        let pagination = Pagination::default();
        let expected_filter = filter.clone();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_roles()
            .withf(move |actual_filter, actual_pagination| {
                *actual_filter == expected_filter && *actual_pagination == pagination
            })
            .times(1)
            .return_once(|_, _| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository).list_roles(&filter, pagination).await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_update_role() {
        let actor_id = UserId::new();
        let id = RoleId::new();
        let name = RoleName::new("editor").unwrap();
        let permissions = vec![Permission::RolesRead, Permission::RolesUpdate];
        let expected_name = name.clone();
        let expected_permissions = permissions.clone();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == actor_id)
            .times(1)
            .return_once(|_| Ok(vec![Permission::RolesRead, Permission::RolesUpdate]));
        repository
            .expect_update_role()
            .withf(move |role| {
                role.id() == id
                    && role.name() == &expected_name
                    && role.permissions() == expected_permissions.as_slice()
            })
            .times(1)
            .return_once(Ok);

        let role = service(repository)
            .update_role(actor_id, id, name, permissions)
            .await
            .unwrap();

        assert_eq!(role.id(), id);
        assert_eq!(role.name().as_str(), "editor");
        assert_eq!(
            role.permissions(),
            &[Permission::RolesRead, Permission::RolesUpdate]
        );
    }

    #[tokio::test]
    async fn should_reject_duplicate_permissions_when_updating_role() {
        let actor_id = UserId::new();
        let id = RoleId::new();
        let name = RoleName::new("editor").unwrap();
        let permissions = vec![Permission::RolesUpdate, Permission::RolesUpdate];

        let mut repository = MockAuthorizationRepository::new();
        repository.expect_update_role().times(0);

        let result = service(repository)
            .update_role(actor_id, id, name, permissions)
            .await;

        assert!(matches!(
            result,
            Err(Error::DuplicatePermission(Permission::RolesUpdate))
        ));
    }

    #[tokio::test]
    async fn should_reject_permissions_the_user_does_not_have_when_updating_role() {
        let actor_id = UserId::new();
        let id = RoleId::new();
        let name = RoleName::new("editor").unwrap();
        let permissions = vec![Permission::RolesUpdate, Permission::RolesDelete];

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == actor_id)
            .times(1)
            .return_once(|_| Ok(vec![Permission::RolesUpdate]));
        repository.expect_update_role().times(0);

        let result = service(repository)
            .update_role(actor_id, id, name, permissions)
            .await;

        assert!(matches!(result, Err(Error::Forbidden)));
    }

    #[tokio::test]
    async fn should_propagate_update_role_error() {
        let actor_id = UserId::new();
        let id = RoleId::new();
        let name = RoleName::new("editor").unwrap();
        let permissions = vec![Permission::RolesUpdate];

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == actor_id)
            .times(1)
            .return_once(|_| Ok(vec![Permission::RolesUpdate]));
        repository
            .expect_update_role()
            .times(1)
            .return_once(|_| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository)
            .update_role(actor_id, id, name, permissions)
            .await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_delete_role() {
        let id = RoleId::new();
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_delete_role()
            .withf(move |actual_id| *actual_id == id)
            .times(1)
            .return_once(|_| Ok(()));

        service(repository).delete_role(id).await.unwrap();
    }

    #[tokio::test]
    async fn should_propagate_delete_role_error() {
        let id = RoleId::new();
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_delete_role()
            .withf(move |actual_id| *actual_id == id)
            .times(1)
            .return_once(|_| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository).delete_role(id).await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_list_user_roles() {
        let user_id = UserId::new();
        let expected = vec![RoleSummary::from_parts(
            RoleId::new(),
            RoleName::new("admin").unwrap(),
        )];
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_roles()
            .withf(move |actual_user_id| *actual_user_id == user_id)
            .times(1)
            .return_once(|_| Ok(expected));

        let roles = service(repository).list_user_roles(user_id).await.unwrap();

        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name().as_str(), "admin");
    }

    #[tokio::test]
    async fn should_propagate_list_user_roles_error() {
        let user_id = UserId::new();
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_roles()
            .withf(move |actual_user_id| *actual_user_id == user_id)
            .times(1)
            .return_once(|_| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository).list_user_roles(user_id).await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_assign_role_to_user() {
        let actor_id = UserId::new();
        let target_user_id = UserId::new();
        let role_id = RoleId::new();
        let role = Role::from_parts(
            role_id,
            RoleName::new("reader").unwrap(),
            vec![Permission::RolesRead],
        );

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_get_role()
            .withf(move |actual_role_id| *actual_role_id == role_id)
            .times(1)
            .return_once(|_| Ok(role));
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == actor_id)
            .times(1)
            .return_once(|_| Ok(vec![Permission::RolesRead]));
        repository
            .expect_assign_role_to_user()
            .withf(move |actual_user_id, actual_role_id| {
                *actual_user_id == target_user_id && *actual_role_id == role_id
            })
            .times(1)
            .return_once(|_, _| Ok(()));

        service(repository)
            .assign_role_to_user(actor_id, target_user_id, role_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_reject_assigning_role_with_permissions_the_user_does_not_have() {
        let actor_id = UserId::new();
        let target_user_id = UserId::new();
        let role_id = RoleId::new();
        let role = Role::from_parts(
            role_id,
            RoleName::new("editor").unwrap(),
            vec![Permission::RolesRead, Permission::RolesDelete],
        );

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_get_role()
            .withf(move |actual_role_id| *actual_role_id == role_id)
            .times(1)
            .return_once(|_| Ok(role));
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == actor_id)
            .times(1)
            .return_once(|_| Ok(vec![Permission::RolesRead]));
        repository.expect_assign_role_to_user().times(0);

        let result = service(repository)
            .assign_role_to_user(actor_id, target_user_id, role_id)
            .await;

        assert!(matches!(result, Err(Error::Forbidden)));
    }

    #[tokio::test]
    async fn should_propagate_assign_role_to_user_error() {
        let actor_id = UserId::new();
        let target_user_id = UserId::new();
        let role_id = RoleId::new();
        let role = Role::from_parts(role_id, RoleName::new("reader").unwrap(), vec![]);

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_get_role()
            .withf(move |actual_role_id| *actual_role_id == role_id)
            .times(1)
            .return_once(|_| Ok(role));
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == actor_id)
            .times(1)
            .return_once(|_| Ok(vec![]));
        repository
            .expect_assign_role_to_user()
            .times(1)
            .return_once(|_, _| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository)
            .assign_role_to_user(actor_id, target_user_id, role_id)
            .await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_unassign_role_from_user() {
        let user_id = UserId::new();
        let role_id = RoleId::new();
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_unassign_role_from_user()
            .withf(move |actual_user_id, actual_role_id| {
                *actual_user_id == user_id && *actual_role_id == role_id
            })
            .times(1)
            .return_once(|_, _| Ok(()));

        service(repository)
            .unassign_role_from_user(user_id, role_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn should_propagate_unassign_role_from_user_error() {
        let user_id = UserId::new();
        let role_id = RoleId::new();
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_unassign_role_from_user()
            .times(1)
            .return_once(|_, _| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository)
            .unassign_role_from_user(user_id, role_id)
            .await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_list_user_permissions() {
        let user_id = UserId::new();
        let expected = vec![Permission::RolesRead, Permission::RolesUpdate];
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == user_id)
            .times(1)
            .return_once({
                let expected = expected.clone();
                move |_| Ok(expected)
            });

        let permissions = service(repository)
            .list_user_permissions(user_id)
            .await
            .unwrap();

        assert_eq!(permissions, expected);
    }

    #[tokio::test]
    async fn should_propagate_list_user_permissions_error() {
        let user_id = UserId::new();
        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_list_user_permissions()
            .withf(move |actual_user_id| *actual_user_id == user_id)
            .times(1)
            .return_once(|_| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository).list_user_permissions(user_id).await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn should_return_true_when_user_has_permission() {
        let user_id = UserId::new();
        let permission = Permission::RolesRead;

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_user_has_permission()
            .withf(move |actual_user_id, actual_permission| {
                *actual_user_id == user_id && *actual_permission == permission
            })
            .times(1)
            .return_once(|_, _| Ok(true));

        let has_permission = service(repository)
            .user_has_permission(user_id, permission)
            .await
            .unwrap();

        assert!(has_permission);
    }

    #[tokio::test]
    async fn should_return_false_when_user_does_not_have_permission() {
        let user_id = UserId::new();
        let permission = Permission::RolesRead;

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_user_has_permission()
            .withf(move |actual_user_id, actual_permission| {
                *actual_user_id == user_id && *actual_permission == permission
            })
            .times(1)
            .return_once(|_, _| Ok(false));

        let has_permission = service(repository)
            .user_has_permission(user_id, permission)
            .await
            .unwrap();

        assert!(!has_permission);
    }

    #[tokio::test]
    async fn should_propagate_user_has_permission_error() {
        let user_id = UserId::new();
        let permission = Permission::RolesRead;

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_user_has_permission()
            .withf(move |actual_user_id, actual_permission| {
                *actual_user_id == user_id && *actual_permission == permission
            })
            .times(1)
            .return_once(|_, _| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository)
            .user_has_permission(user_id, permission)
            .await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }
}
