use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserActivityRepository: Send + Sync {
    async fn record_page_activity(&self, user_id: Uuid, page_key: &str) -> anyhow::Result<()>;
}
