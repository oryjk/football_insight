use async_trait::async_trait;
use uuid::Uuid;

use crate::push_notification::domain::device_token::DeviceToken;

#[async_trait]
pub trait DeviceTokenRepository: Send + Sync {
    async fn upsert(&self, user_id: Uuid, device_token: &str, platform: &str)
    -> anyhow::Result<()>;
    async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<DeviceToken>>;
    async fn list_by_users(&self, user_ids: &[Uuid]) -> anyhow::Result<Vec<DeviceToken>>;
    async fn delete(&self, device_token: &str) -> anyhow::Result<()>;
}
