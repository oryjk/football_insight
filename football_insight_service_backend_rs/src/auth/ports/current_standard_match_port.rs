use async_trait::async_trait;

#[async_trait]
pub trait CurrentStandardMatchPort: Send + Sync {
    async fn fetch_current_match_id(&self) -> anyhow::Result<Option<String>>;
}
