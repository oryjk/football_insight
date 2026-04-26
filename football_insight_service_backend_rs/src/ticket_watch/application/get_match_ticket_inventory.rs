use std::sync::Arc;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchInventoryEntry, ports::ticket_monitor_port::TicketMonitorPort,
};

pub struct GetMatchTicketInventoryUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl GetMatchTicketInventoryUseCase {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }

    pub async fn execute(&self, match_id: i64) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
        self.ticket_monitor_port
            .fetch_inventory(match_id, None, None)
            .await
    }

    pub async fn execute_with_since(
        &self,
        match_id: i64,
        fallback_match_id: Option<i64>,
        since: Option<&str>,
    ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
        self.ticket_monitor_port
            .fetch_inventory(match_id, fallback_match_id, since)
            .await
    }
}
