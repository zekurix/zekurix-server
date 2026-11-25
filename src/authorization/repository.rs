use async_trait::async_trait;

use crate::error::Result;
use crate::router::{Page, PaginationQuery};
use crate::user::UserId;

use super::Permission;
use super::role::{Role, RoleFilter, RoleId, RoleSummary};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthorizationRepository: Send + Sync {
    async fn list_permissions(&self) -> Result<Vec<Permission>>;
    async fn create_role(&self, role: Role) -> Result<Role>;
    async fn get_role(&self, id: RoleId) -> Result<Role>;
    async fn list_roles(
        &self,
        filter: &RoleFilter,
        pagination: PaginationQuery,
    ) -> Result<Page<Role>>;
    async fn update_role(&self, role: Role) -> Result<Role>;
    async fn delete_role(&self, id: RoleId) -> Result<()>;
    async fn list_user_roles(&self, user_id: UserId) -> Result<Vec<RoleSummary>>;
    async fn assign_role_to_user(&self, user_id: UserId, role_id: RoleId) -> Result<()>;
    async fn unassign_role_from_user(&self, user_id: UserId, role_id: RoleId) -> Result<()>;
    async fn list_user_permissions(&self, user_id: UserId) -> Result<Vec<Permission>>;
    async fn user_has_permission(&self, user_id: UserId, permission: Permission) -> Result<bool>;
}
