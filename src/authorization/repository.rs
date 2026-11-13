use async_trait::async_trait;

use crate::error::Result;
use crate::router::Pagination;
use crate::user::UserId;

use super::{Permission, Role, RoleId, RoleSummary};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthorizationRepository: Send + Sync {
    async fn list_permissions(&self) -> Result<Vec<Permission>>;
    async fn create_role(&self, role: Role) -> Result<Role>;
    async fn get_role(&self, id: RoleId) -> Result<Role>;
    async fn list_roles(&self, pagination: Pagination) -> Result<Vec<Role>>;
    async fn update_role(&self, role: Role) -> Result<Role>;
    async fn delete_role(&self, id: RoleId) -> Result<()>;
    async fn get_user_roles(&self, user_id: UserId) -> Result<Vec<RoleSummary>>;
    async fn assign_role_to_user(&self, user_id: UserId, role_id: RoleId) -> Result<()>;
    async fn unassign_role_from_user(&self, user_id: UserId, role_id: RoleId) -> Result<()>;
    async fn get_user_permissions(&self, user_id: UserId) -> Result<Vec<Permission>>;
}
