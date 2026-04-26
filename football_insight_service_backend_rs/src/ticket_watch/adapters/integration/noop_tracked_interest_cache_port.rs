use async_trait::async_trait;
use uuid::Uuid;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchTrackedInterest,
    ports::tracked_interest_cache_port::TrackedInterestCachePort,
};

pub struct NoopTrackedInterestCachePort;

#[async_trait]
impl TrackedInterestCachePort for NoopTrackedInterestCachePort {
    async fn get(
        &self,
        _match_id: i64,
        _user_id: Uuid,
    ) -> anyhow::Result<Option<Vec<TicketWatchTrackedInterest>>> {
        Ok(None)
    }

    async fn set(
        &self,
        _match_id: i64,
        _user_id: Uuid,
        _tracked_interests: &[TicketWatchTrackedInterest],
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete(&self, _match_id: i64, _user_id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}
