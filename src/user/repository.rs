use async_trait::async_trait;

use crate::error::Result;
use crate::identity::Identity;
use crate::router::Pagination;

use super::{User, UserId};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: User, identity: Identity) -> Result<User>;
    async fn find(&self, id: UserId) -> Result<User>;
    async fn list(&self, pagination: Pagination) -> Result<Vec<User>>;
    async fn delete(&self, id: UserId) -> Result<()>;
}
