use super::domain::*;
use async_trait::async_trait;
#[async_trait]
pub trait OutlookRepository: Send + Sync {
    async fn find(&self, match_id: i64) -> Result<Option<Outlook>, ReadError>;
    async fn record(&self, season: i32) -> Result<Record, ReadError>;
}
