use std::sync::Arc;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchInventoryEntry, ports::ticket_monitor_port::TicketMonitorPort,
};

pub struct GetYukunTicketInventoryUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl GetYukunTicketInventoryUseCase {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }

    pub async fn execute(
        &self,
        match_id: i64,
        since: Option<&str>,
    ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
        let normalized_since = since
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "sale_start_at is required to build inventory since; refusing full inventory query"
                )
            })?;

        let (inventory, _) = self
            .ticket_monitor_port
            .fetch_yukun_reflux(match_id, Some(normalized_since))
            .await?;

        Ok(inventory)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::GetYukunTicketInventoryUseCase;
    use crate::ticket_watch::{
        domain::ticket_watch::{
            TicketWatchBlockInterest, TicketWatchCurrentMatchView, TicketWatchInventoryEntry,
            TicketWatchMatchSummary, TicketWatchRegion, TicketWatchTrackedInterest,
        },
        ports::ticket_monitor_port::TicketMonitorPort,
    };

    #[derive(Default)]
    struct StubTicketMonitorPort {
        yukun_reflux_calls: Mutex<Vec<(i64, Option<String>)>>,
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
            _block_name: &str,
        ) -> anyhow::Result<TicketWatchBlockInterest> {
            unreachable!()
        }

        async fn fetch_yukun_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
            unreachable!()
        }

        async fn fetch_yukun_current_match(
            &self,
        ) -> anyhow::Result<crate::ticket_watch::domain::ticket_watch::TicketWatchCurrentMatchView>
        {
            unreachable!()
        }

        async fn fetch_yukun_reflux(
            &self,
            match_id: i64,
            since: Option<&str>,
        ) -> anyhow::Result<(Vec<TicketWatchInventoryEntry>, Vec<TicketWatchRegion>)> {
            self.yukun_reflux_calls
                .lock()
                .expect("calls")
                .push((match_id, since.map(str::to_string)));
            Ok((vec![], vec![]))
        }
    }

    #[tokio::test]
    async fn execute_rejects_missing_since() {
        let port = Arc::new(StubTicketMonitorPort::default());
        let use_case = GetYukunTicketInventoryUseCase::new(port.clone());

        let error = use_case
            .execute(288651, None)
            .await
            .expect_err("missing since should fail");

        assert!(error.to_string().contains("sale_start_at"));
        assert!(port.yukun_reflux_calls.lock().expect("calls").is_empty());
    }

    #[tokio::test]
    async fn execute_forwards_non_empty_since() {
        let port = Arc::new(StubTicketMonitorPort::default());
        let use_case = GetYukunTicketInventoryUseCase::new(port.clone());

        use_case
            .execute(288651, Some(" 2026-05-04T14:10:00+08:00 "))
            .await
            .expect("execute");

        assert_eq!(
            port.yukun_reflux_calls.lock().expect("calls").as_slice(),
            &[(288651, Some("2026-05-04T14:10:00+08:00".to_string()))],
        );
    }
}
