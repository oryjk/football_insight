use std::sync::Arc;

use uuid::Uuid;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchBlockInterest, ports::ticket_monitor_port::TicketMonitorPort,
};

pub struct GetMatchBlockInterestsUseCase {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl GetMatchBlockInterestsUseCase {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }

    pub async fn execute(
        &self,
        match_id: i64,
        viewer_user_id: Option<Uuid>,
    ) -> anyhow::Result<Vec<TicketWatchBlockInterest>> {
        self.ticket_monitor_port
            .fetch_block_interests(match_id, viewer_user_id)
            .await
    }
}
