use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::auth_license::domain::license::UserLicense;

#[async_trait]
pub trait LicenseRepository: Send + Sync {
    async fn create_license(
        &self,
        user_id: Uuid,
        license_key: &str,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<UserLicense>;
    async fn find_by_key(&self, license_key: &str) -> anyhow::Result<Option<UserLicense>>;
    async fn mark_used(&self, license_id: i64) -> anyhow::Result<()>;
}
