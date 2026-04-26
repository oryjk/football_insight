use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserMembershipPort: Send + Sync {
    async fn get_user_open_id(&self, user_id: Uuid) -> anyhow::Result<Option<String>>;
    async fn get_user_membership_tier(&self, user_id: Uuid) -> anyhow::Result<Option<String>>;
    async fn update_user_membership_tier(&self, user_id: Uuid, tier: &str) -> anyhow::Result<()>;
}
