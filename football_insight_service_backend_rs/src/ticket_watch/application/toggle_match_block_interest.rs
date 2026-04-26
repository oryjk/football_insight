use std::sync::Arc;

use uuid::Uuid;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchBlockInterest, ports::ticket_monitor_port::TicketMonitorPort,
    ports::tracked_interest_cache_port::TrackedInterestCachePort,
};

pub struct ToggleMatchBlockInterestUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
    tracked_interest_cache_port: Arc<dyn TrackedInterestCachePort>,
}

impl ToggleMatchBlockInterestUseCase {
    pub fn new(
        ticket_monitor_port: Arc<dyn TicketMonitorPort>,
        tracked_interest_cache_port: Arc<dyn TrackedInterestCachePort>,
    ) -> Self {
        Self {
            ticket_monitor_port,
            tracked_interest_cache_port,
        }
    }

    pub async fn execute(
        &self,
        match_id: i64,
        user_id: Uuid,
        block_name: &str,
    ) -> anyhow::Result<TicketWatchBlockInterest> {
        let interest = self
            .ticket_monitor_port
            .toggle_block_interest(match_id, user_id, block_name)
            .await?;

        if let Err(error) = self
            .tracked_interest_cache_port
            .delete(match_id, user_id)
            .await
        {
            tracing::warn!(
                error = %error,
                match_id,
                user_id = %user_id,
                block_name,
                "failed to delete tracked interests redis cache after toggle"
            );
        }

        Ok(interest)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::ToggleMatchBlockInterestUseCase;
    use crate::ticket_watch::{
        domain::ticket_watch::{
            TicketWatchBlockInterest, TicketWatchCurrentMatchView, TicketWatchInventoryEntry,
            TicketWatchMatchSummary, TicketWatchRegion, TicketWatchTrackedInterest,
        },
        ports::{
            ticket_monitor_port::TicketMonitorPort,
            tracked_interest_cache_port::TrackedInterestCachePort,
        },
    };

    struct StubTicketMonitorPort;

    #[async_trait]
    impl TicketMonitorPort for StubTicketMonitorPort {
        async fn fetch_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
            unreachable!()
        }

        async fn fetch_all_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
            unreachable!()
        }

        async fn fetch_regions(&self) -> anyhow::Result<Vec<TicketWatchRegion>> {
            unreachable!()
        }

        async fn fetch_inventory(
            &self,
            _match_id: i64,
            _fallback_match_id: Option<i64>,
            _since: Option<&str>,
        ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
            unreachable!()
        }

        async fn fetch_block_interests(
            &self,
            _match_id: i64,
            _viewer_user_id: Option<Uuid>,
        ) -> anyhow::Result<Vec<TicketWatchBlockInterest>> {
            unreachable!()
        }

        async fn fetch_tracked_interests(
            &self,
            _match_id: i64,
            _user_id: Uuid,
        ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>> {
            unreachable!()
        }

        async fn toggle_block_interest(
            &self,
            _match_id: i64,
            _user_id: Uuid,
            block_name: &str,
        ) -> anyhow::Result<TicketWatchBlockInterest> {
            Ok(TicketWatchBlockInterest {
                block_name: block_name.to_string(),
                interested_user_count: 1,
                viewer_interested: true,
            })
        }
    }

    struct StubTrackedInterestCachePort {
        delete_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TrackedInterestCachePort for StubTrackedInterestCachePort {
        async fn get(
            &self,
            _match_id: i64,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<Vec<TicketWatchTrackedInterest>>> {
            unreachable!()
        }

        async fn set(
            &self,
            _match_id: i64,
            _user_id: Uuid,
            _tracked_interests: &[TicketWatchTrackedInterest],
        ) -> anyhow::Result<()> {
            unreachable!()
        }

        async fn delete(&self, _match_id: i64, _user_id: Uuid) -> anyhow::Result<()> {
            self.delete_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn invalidates_tracked_interest_cache_after_toggle() {
        let delete_calls = Arc::new(AtomicUsize::new(0));
        let use_case = ToggleMatchBlockInterestUseCase::new(
            Arc::new(StubTicketMonitorPort),
            Arc::new(StubTrackedInterestCachePort {
                delete_calls: delete_calls.clone(),
            }),
        );

        let result = use_case
            .execute(573, Uuid::nil(), "104")
            .await
            .expect("toggle result");

        assert_eq!(result.block_name, "104");
        assert_eq!(delete_calls.load(Ordering::SeqCst), 1);
    }
}
