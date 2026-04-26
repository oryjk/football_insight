use async_trait::async_trait;
use sqlx::PgPool;

use crate::health::{domain::health_status::HealthStatus, ports::health_port::HealthPort};

#[derive(Clone)]
pub struct PostgresHealthPort {
    pool: PgPool,
}

impl PostgresHealthPort {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthPort for PostgresHealthPort {
    async fn get_health_status(&self) -> anyhow::Result<HealthStatus> {
        let _value: i32 = sqlx::query_scalar("SELECT 1").fetch_one(&self.pool).await?;
        Ok(HealthStatus::ok())
    }
}
