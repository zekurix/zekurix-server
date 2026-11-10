use std::collections::HashSet;
use std::sync::Arc;

use crate::error::{Error, Result};

use super::{AuthorizationRepository, Permission, Role, RoleName};

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

    pub async fn create_role(&self, name: RoleName, permissions: Vec<Permission>) -> Result<Role> {
        let mut seen = HashSet::new();
        for permission in &permissions {
            if !seen.insert(permission) {
                return Err(Error::DuplicatePermission(*permission));
            }
        }

        let role = Role::new(name, permissions);

        self.repository.create_role(role).await
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
        let name = RoleName::new("admin").unwrap();
        let permissions = vec![Permission::RolesRead, Permission::RolesUpdate];
        let expected_name = name.clone();
        let expected_permissions = permissions.clone();
        let expected_name_for_match = expected_name.clone();
        let expected_permissions_for_match = expected_permissions.clone();

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_create_role()
            .withf(move |role| {
                role.name() == &expected_name_for_match
                    && role.permissions() == expected_permissions_for_match.as_slice()
            })
            .times(1)
            .return_once(|role| Ok(role));

        let role = service(repository)
            .create_role(name, permissions)
            .await
            .unwrap();

        assert_eq!(role.name(), &expected_name);
        assert_eq!(role.permissions(), expected_permissions.as_slice());
    }

    #[tokio::test]
    async fn should_reject_duplicate_permissions_when_creating_role() {
        let name = RoleName::new("admin").unwrap();
        let permissions = vec![Permission::RolesRead, Permission::RolesRead];

        let mut repository = MockAuthorizationRepository::new();
        repository.expect_create_role().times(0);

        let result = service(repository).create_role(name, permissions).await;

        assert!(matches!(
            result,
            Err(Error::DuplicatePermission(Permission::RolesRead))
        ));
    }

    #[tokio::test]
    async fn should_propagate_create_role_error() {
        let name = RoleName::new("admin").unwrap();
        let permissions = vec![Permission::RolesRead];

        let mut repository = MockAuthorizationRepository::new();
        repository
            .expect_create_role()
            .times(1)
            .return_once(|_| Err(Error::InternalError(anyhow::anyhow!("error"))));

        let result = service(repository).create_role(name, permissions).await;

        assert!(matches!(result, Err(Error::InternalError(_))));
    }
}
