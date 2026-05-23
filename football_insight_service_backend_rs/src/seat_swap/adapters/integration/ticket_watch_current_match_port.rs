use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    seat_swap::{
        domain::{SeatSwapCurrentMatch, SeatSwapRegion},
        ports::current_match_port::CurrentSeatSwapMatchPort,
    },
    ticket_watch::{
        application::list_ticket_watch_regions::complete_phoenix_hill_regions,
        ports::ticket_monitor_port::TicketMonitorPort,
    },
};

pub struct TicketWatchCurrentSeatSwapMatchPort {
    ticket_monitor_port: Arc<dyn TicketMonitorPort>,
}

impl TicketWatchCurrentSeatSwapMatchPort {
    pub fn new(ticket_monitor_port: Arc<dyn TicketMonitorPort>) -> Self {
        Self {
            ticket_monitor_port,
        }
    }
}

#[async_trait]
impl CurrentSeatSwapMatchPort for TicketWatchCurrentSeatSwapMatchPort {
    async fn current_match(&self) -> anyhow::Result<Option<SeatSwapCurrentMatch>> {
        let view = self.ticket_monitor_port.fetch_current_match().await?;
        Ok(view.current_match.and_then(|item| {
            let involves_chengdu = item.home_team_name.contains("成都蓉城")
                || item.away_team_name.contains("成都蓉城");
            if !item.is_current || !involves_chengdu {
                return None;
            }
            Some(SeatSwapCurrentMatch {
                match_id: item.match_id,
                home_team_name: item.home_team_name,
                away_team_name: item.away_team_name,
                kickoff_at: item.kickoff_at,
            })
        }))
    }

    async fn current_regions(&self) -> anyhow::Result<Vec<SeatSwapRegion>> {
        let regions =
            complete_phoenix_hill_regions(self.ticket_monitor_port.fetch_regions().await?);

        Ok(regions
            .into_iter()
            .map(|region| SeatSwapRegion {
                region_key: region.block_key,
                region_name: region.block_name,
            })
            .collect())
    }
}
