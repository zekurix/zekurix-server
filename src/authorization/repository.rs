use async_trait::async_trait;

use crate::error::Result;
use crate::router::Pagination;

use super::{Permission, Role, RoleId};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthorizationRepository: Send + Sync {
    async fn list_permissions(&self) -> Result<Vec<Permission>>;
    async fn create_role(&self, role: Role) -> Result<Role>;
    async fn get_role(&self, id: RoleId) -> Result<Role>;
    async fn list_roles(&self, pagination: Pagination) -> Result<Vec<Role>>;
    async fn update_role(&self, role: Role) -> Result<Role>;
    async fn delete_role(&self, id: RoleId) -> Result<()>;
}
