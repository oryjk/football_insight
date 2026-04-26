use std::sync::Arc;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchCurrentMatchView,
    ports::ticket_monitor_port::TicketMonitorPort,
};

pub struct GetCurrentTicketWatchMatchUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl GetCurrentTicketWatchMatchUseCase {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }

    pub async fn execute(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
        self.ticket_monitor_port.fetch_current_match().await
    }
}
