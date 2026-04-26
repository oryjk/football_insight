use std::sync::Arc;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchMatchSummary, ports::ticket_monitor_port::TicketMonitorPort,
};

pub struct ListTicketWatchMatchesUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl ListTicketWatchMatchesUseCase {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }

    pub async fn execute(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
        self.ticket_monitor_port.fetch_all_matches().await
    }
}
