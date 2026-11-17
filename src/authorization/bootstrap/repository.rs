use async_trait::async_trait;

use crate::authorization::RoleName;
use crate::error::Result;
use crate::identity::Identity;
use crate::user::User;

#[async_trait]
pub trait AuthorizationBootstrapRepository: Send + Sync {
    async fn begin(&self) -> Result<Box<dyn AuthorizationBootstrapTransaction + '_>>;
}

#[async_trait]
pub trait AuthorizationBootstrapTransaction: Send {
    async fn is_completed(&mut self, name: &str) -> Result<bool>;
    async fn create_super_admin(&mut self, user: User, identity: Identity, role_name: RoleName) -> Result<()>;
    async fn mark_completed(&mut self, name: &str) -> Result<()>;
    async fn commit(self: Box<Self>) -> Result<()>;
}
