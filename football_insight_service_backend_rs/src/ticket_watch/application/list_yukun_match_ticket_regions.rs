use std::sync::Arc;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchRegion, ports::ticket_monitor_port::TicketMonitorPort,
};

pub struct ListYukunMatchTicketRegionsUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl ListYukunMatchTicketRegionsUseCase {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }

    pub async fn execute(
        &self,
        match_id: i64,
        since: Option<&str>,
    ) -> anyhow::Result<Vec<TicketWatchRegion>> {
        let normalized_since = since
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "sale_start_at is required to build inventory since; refusing full inventory query"
                )
            })?;

        let (_, regions) = self
            .ticket_monitor_port
            .fetch_yukun_reflux(match_id, Some(normalized_since))
            .await?;

        Ok(regions)
    }
}
