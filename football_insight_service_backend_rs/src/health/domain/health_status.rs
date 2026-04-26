use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub service: &'static str,
    pub status: &'static str,
    pub timestamp: DateTime<Utc>,
}

impl HealthStatus {
    pub fn ok() -> Self {
        Self {
            service: "football_insight_service_backend_rs",
            status: "ok",
            timestamp: Utc::now(),
        }
    }
}
