use std::sync::Arc;

use crate::error::Result;

use super::{AuthorizationRepository, Permission};

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
}
