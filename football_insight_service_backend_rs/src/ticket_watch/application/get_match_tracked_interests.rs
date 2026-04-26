use std::sync::Arc;

use uuid::Uuid;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchTrackedInterest,
    ports::{
        ticket_monitor_port::TicketMonitorPort,
        tracked_interest_cache_port::TrackedInterestCachePort,
    },
};

pub struct GetMatchTrackedInterestsUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
    tracked_interest_cache_port: Arc<dyn TrackedInterestCachePort>,
}

impl GetMatchTrackedInterestsUseCase {
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
    ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>> {
        match self
            .tracked_interest_cache_port
            .get(match_id, user_id)
            .await
        {
            Ok(Some(cached)) => return Ok(cached),
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    match_id,
                    user_id = %user_id,
                    "failed to read tracked interests from redis cache"
                );
            }
        }

        let tracked_interests = self
            .ticket_monitor_port
            .fetch_tracked_interests(match_id, user_id)
            .await?;

        if !tracked_interests.is_empty() {
            if let Err(error) = self
                .tracked_interest_cache_port
                .set(match_id, user_id, &tracked_interests)
                .await
            {
                tracing::warn!(
                    error = %error,
                    match_id,
                    user_id = %user_id,
                    "failed to write tracked interests to redis cache"
                );
            }
        }

        Ok(tracked_interests)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::GetMatchTrackedInterestsUseCase;
    use crate::ticket_watch::{
        domain::ticket_watch::TicketWatchTrackedInterest,
        ports::{
            ticket_monitor_port::TicketMonitorPort,
            tracked_interest_cache_port::TrackedInterestCachePort,
        },
    };
    use async_trait::async_trait;
    use uuid::Uuid;

    struct StubTicketMonitorPort {
        calls: Arc<AtomicUsize>,
        response: Vec<TicketWatchTrackedInterest>,
    }

    #[async_trait]
    impl TicketMonitorPort for StubTicketMonitorPort {
        async fn fetch_current_match(
            &self,
        ) -> anyhow::Result<crate::ticket_watch::domain::ticket_watch::TicketWatchCurrentMatchView>
        {
            unreachable!()
        }

        async fn fetch_all_matches(
            &self,
        ) -> anyhow::Result<Vec<crate::ticket_watch::domain::ticket_watch::TicketWatchMatchSummary>>
        {
            unreachable!()
        }

        async fn fetch_regions(
            &self,
        ) -> anyhow::Result<Vec<crate::ticket_watch::domain::ticket_watch::TicketWatchRegion>>
        {
            unreachable!()
        }

        async fn fetch_inventory(
            &self,
            _match_id: i64,
            _fallback_match_id: Option<i64>,
            _since: Option<&str>,
        ) -> anyhow::Result<Vec<crate::ticket_watch::domain::ticket_watch::TicketWatchInventoryEntry>>
        {
            unreachable!()
        }

        async fn fetch_block_interests(
            &self,
            _match_id: i64,
            _viewer_user_id: Option<Uuid>,
        ) -> anyhow::Result<Vec<crate::ticket_watch::domain::ticket_watch::TicketWatchBlockInterest>>
        {
            unreachable!()
        }

        async fn fetch_tracked_interests(
            &self,
            _match_id: i64,
            _user_id: Uuid,
        ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(self.response.clone())
        }

        async fn toggle_block_interest(
            &self,
            _match_id: i64,
            _user_id: Uuid,
            _block_name: &str,
        ) -> anyhow::Result<crate::ticket_watch::domain::ticket_watch::TicketWatchBlockInterest>
        {
            unreachable!()
        }
    }

    struct StubTrackedInterestCachePort {
        get_response: Option<Vec<TicketWatchTrackedInterest>>,
        set_calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl TrackedInterestCachePort for StubTrackedInterestCachePort {
        async fn get(
            &self,
            _match_id: i64,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<Vec<TicketWatchTrackedInterest>>> {
            Ok(self.get_response.clone())
        }

        async fn set(
            &self,
            _match_id: i64,
            _user_id: Uuid,
            _tracked_interests: &[TicketWatchTrackedInterest],
        ) -> anyhow::Result<()> {
            self.set_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn delete(&self, _match_id: i64, _user_id: Uuid) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn returns_cached_tracked_interests_without_calling_ticket_monitor() {
        let user_id = Uuid::nil();
        let fetch_calls = Arc::new(AtomicUsize::new(0));
        let set_calls = Arc::new(AtomicUsize::new(0));
        let cached = vec![TicketWatchTrackedInterest {
            block_name: "104".to_string(),
            started_at: "2026-04-20T12:00:00+08:00".to_string(),
            first_inventory_at: Some("2026-04-20T12:08:00+08:00".to_string()),
        }];
        let use_case = GetMatchTrackedInterestsUseCase::new(
            Arc::new(StubTicketMonitorPort {
                calls: fetch_calls.clone(),
                response: vec![],
            }),
            Arc::new(StubTrackedInterestCachePort {
                get_response: Some(cached.clone()),
                set_calls: set_calls.clone(),
            }),
        );

        let result = use_case.execute(573, user_id).await.expect("result");

        assert_eq!(result.len(), 1);
        assert_eq!(fetch_calls.load(Ordering::SeqCst), 0);
        assert_eq!(set_calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn caches_non_empty_ticket_monitor_response() {
        let user_id = Uuid::nil();
        let fetch_calls = Arc::new(AtomicUsize::new(0));
        let set_calls = Arc::new(AtomicUsize::new(0));
        let upstream = vec![TicketWatchTrackedInterest {
            block_name: "105".to_string(),
            started_at: "2026-04-20T12:01:00+08:00".to_string(),
            first_inventory_at: None,
        }];
        let use_case = GetMatchTrackedInterestsUseCase::new(
            Arc::new(StubTicketMonitorPort {
                calls: fetch_calls.clone(),
                response: upstream.clone(),
            }),
            Arc::new(StubTrackedInterestCachePort {
                get_response: None,
                set_calls: set_calls.clone(),
            }),
        );

        let result = use_case.execute(573, user_id).await.expect("result");

        assert_eq!(result.len(), 1);
        assert_eq!(fetch_calls.load(Ordering::SeqCst), 1);
        assert_eq!(set_calls.load(Ordering::SeqCst), 1);
    }
}
