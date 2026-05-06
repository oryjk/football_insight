use std::sync::Arc;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchCurrentMatchView,
    ports::ticket_monitor_port::TicketMonitorPort,
};

pub struct GetYukunCurrentTicketWatchMatchUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl GetYukunCurrentTicketWatchMatchUseCase {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }

    pub async fn execute(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
        self.ticket_monitor_port.fetch_yukun_current_match().await
    }
}
