use async_trait::async_trait;
use uuid::Uuid;

use crate::ticket_watch::domain::ticket_watch::TicketWatchTrackedInterest;

#[async_trait]
pub trait TrackedInterestCachePort: Send + Sync {
    async fn get(
        &self,
        match_id: i64,
        user_id: Uuid,
    ) -> anyhow::Result<Option<Vec<TicketWatchTrackedInterest>>>;

    async fn set(
        &self,
        match_id: i64,
        user_id: Uuid,
        tracked_interests: &[TicketWatchTrackedInterest],
    ) -> anyhow::Result<()>;

    async fn delete(&self, match_id: i64, user_id: Uuid) -> anyhow::Result<()>;
}
