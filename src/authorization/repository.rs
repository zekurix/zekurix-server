use async_trait::async_trait;

use crate::error::Result;

use super::Permission;
use super::Role;
use super::RoleName;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthorizationRepository: Send + Sync {
    async fn list_permissions(&self) -> Result<Vec<Permission>>;
    async fn create_role(&self, name: RoleName, permissions: Vec<Permission>) -> Result<Role>;
}
