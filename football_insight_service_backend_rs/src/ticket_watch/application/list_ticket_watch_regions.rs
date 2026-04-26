use std::sync::Arc;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchRegion, ports::ticket_monitor_port::TicketMonitorPort,
};

pub struct ListTicketWatchRegionsUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl ListTicketWatchRegionsUseCase {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }

    pub async fn execute(&self) -> anyhow::Result<Vec<TicketWatchRegion>> {
        self.ticket_monitor_port.fetch_regions().await
    }
}
