use std::sync::Arc;

use anyhow::anyhow;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchInventoryEntry, ports::ticket_monitor_port::TicketMonitorPort,
};

pub struct GetMatchTicketInventoryUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::GetMatchTicketInventoryUseCase;
    use crate::ticket_watch::{
        domain::ticket_watch::{
            TicketWatchBlockInterest, TicketWatchCurrentMatchView, TicketWatchInventoryEntry,
            TicketWatchMatchSummary, TicketWatchRegion, TicketWatchTrackedInterest,
        },
        ports::ticket_monitor_port::TicketMonitorPort,
    };

    #[derive(Default)]
    struct StubTicketMonitorPort {
        inventory_calls: Mutex<Vec<(i64, Option<i64>, Option<String>)>>,
    }

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
            match_id: i64,
            fallback_match_id: Option<i64>,
            since: Option<&str>,
        ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
            self.inventory_calls.lock().expect("calls").push((
                match_id,
                fallback_match_id,
                since.map(str::to_string),
            ));
            Ok(vec![])
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
            _block_name: &str,
        ) -> anyhow::Result<TicketWatchBlockInterest> {
            unreachable!()
        }
        async fn fetch_yukun_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
            Ok(vec![])
        }

        async fn fetch_yukun_current_match(
            &self,
        ) -> anyhow::Result<crate::ticket_watch::domain::ticket_watch::TicketWatchCurrentMatchView>
        {
            Ok(
                crate::ticket_watch::domain::ticket_watch::TicketWatchCurrentMatchView {
                    current_match: None,
                    group_ticket_active: false,
                    message: "".to_string(),
                },
            )
        }

        async fn fetch_yukun_reflux(
            &self,
            _match_id: i64,
            _since: Option<&str>,
        ) -> anyhow::Result<(Vec<TicketWatchInventoryEntry>, Vec<TicketWatchRegion>)> {
            Ok((vec![], vec![]))
        }
    }

    #[tokio::test]
    async fn execute_with_since_rejects_missing_since() {
        let port = Arc::new(StubTicketMonitorPort::default());
        let use_case = GetMatchTicketInventoryUseCase::new(port.clone());

        let error = use_case
            .execute_with_since(574, Some(78), None)
            .await
            .expect_err("missing since should fail");

        assert!(error.to_string().contains("sale_start_at"));
        assert!(port.inventory_calls.lock().expect("calls").is_empty());
    }

    #[tokio::test]
    async fn execute_with_since_rejects_blank_since() {
        let port = Arc::new(StubTicketMonitorPort::default());
        let use_case = GetMatchTicketInventoryUseCase::new(port.clone());

        let error = use_case
            .execute_with_since(574, Some(78), Some("   "))
            .await
            .expect_err("blank since should fail");

        assert!(error.to_string().contains("sale_start_at"));
        assert!(port.inventory_calls.lock().expect("calls").is_empty());
    }
}

impl GetMatchTicketInventoryUseCase {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }

    pub async fn execute(&self, match_id: i64) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
        self.execute_with_since(match_id, None, None).await
    }

    pub async fn execute_with_since(
        &self,
        match_id: i64,
        fallback_match_id: Option<i64>,
        since: Option<&str>,
    ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
        let since = since
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                anyhow!("sale_start_at is required to build inventory since; refusing full inventory query")
            })?;

        self.ticket_monitor_port
            .fetch_inventory(match_id, fallback_match_id, Some(since))
            .await
    }
}
