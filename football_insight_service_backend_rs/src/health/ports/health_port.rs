use async_trait::async_trait;

use crate::health::domain::health_status::HealthStatus;

#[async_trait]
pub trait HealthPort: Send + Sync {
    async fn get_health_status(&self) -> anyhow::Result<HealthStatus>;
}
