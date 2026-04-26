use std::sync::Arc;

use crate::health::{domain::health_status::HealthStatus, ports::health_port::HealthPort};

pub struct GetHealthUseCase {
    health_port: Arc<dyn HealthPort>,
}

impl GetHealthUseCase {
    pub fn new(health_port: Arc<dyn HealthPort>) -> Self {
        Self { health_port }
    }

    pub async fn execute(&self) -> anyhow::Result<HealthStatus> {
        self.health_port.get_health_status().await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::GetHealthUseCase;
    use crate::health::{domain::health_status::HealthStatus, ports::health_port::HealthPort};

    struct FakeHealthPort;

    #[async_trait]
    impl HealthPort for FakeHealthPort {
        async fn get_health_status(&self) -> anyhow::Result<HealthStatus> {
            Ok(HealthStatus::ok())
        }
    }

    #[tokio::test]
    async fn execute_returns_health_status_from_port() {
        let use_case = GetHealthUseCase::new(Arc::new(FakeHealthPort));
        let status = use_case
            .execute()
            .await
            .expect("health status should resolve");

        assert_eq!(status.service, "football_insight_service_backend_rs");
        assert_eq!(status.status, "ok");
    }
}
