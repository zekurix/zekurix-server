use async_trait::async_trait;

use crate::error::Result;

use super::Permission;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthorizationRepository: Send + Sync {
    async fn list_permissions(&self) -> Result<Vec<Permission>>;
}
