use async_trait::async_trait;

use crate::error::Result;

use super::{User, UserId};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find(&self, id: UserId) -> Result<User>;
    async fn create(&self, user: User) -> Result<()>;
}
