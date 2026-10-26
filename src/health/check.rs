use async_trait::async_trait;

use super::HealthStatus;

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn health(&self) -> HealthStatus;
}
