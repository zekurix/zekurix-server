use async_trait::async_trait;

use crate::error::Result;

use super::{Identity, Issuer, Subject};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait IdentityRepository: Send + Sync {
    async fn find(&self, issuer: &Issuer, subject: &Subject) -> Result<Identity>;
}
