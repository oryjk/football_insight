use std::sync::Arc;

use crate::ticket_watch::{
    domain::ticket_watch::TicketWatchRegion, ports::ticket_monitor_port::TicketMonitorPort,
};

const PHOENIX_HILL_REGION_BLOCKS: &[&str] = &[
    "101", "102", "103", "104", "105", "106", "107", "108", "109", "110", "111", "112", "113",
    "114", "115", "116", "117", "118", "119", "120", "121", "122", "123", "124", "125", "126",
    "127", "128", "129", "130", "131", "132", "501", "502", "503", "504", "505", "506", "507",
    "508", "509", "510", "511", "512", "513", "514", "515", "516", "517", "518", "519", "520",
    "521", "522", "523", "524", "525", "526", "527", "528", "529", "530", "531", "532", "533",
    "534", "535", "536",
];

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
        let regions = self.ticket_monitor_port.fetch_regions().await?;
        Ok(complete_phoenix_hill_regions(regions))
    }
}

pub fn complete_phoenix_hill_regions(
    mut regions: Vec<TicketWatchRegion>,
) -> Vec<TicketWatchRegion> {
    for block in PHOENIX_HILL_REGION_BLOCKS {
        if regions.iter().any(|region| region.block_key == *block) {
            continue;
        }

        regions.push(TicketWatchRegion {
            block_key: (*block).to_string(),
            block_name: (*block).to_string(),
            price: String::new(),
            usable_count: 0,
            estate: 0,
        });
    }

    regions.sort_by(|left, right| left.block_key.cmp(&right.block_key));
    regions
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    use crate::ticket_watch::{
        domain::ticket_watch::{
            TicketWatchBlockInterest, TicketWatchCurrentMatchView, TicketWatchInventoryEntry,
            TicketWatchMatchSummary, TicketWatchTrackedInterest,
        },
        ports::ticket_monitor_port::TicketMonitorPort,
    };
    use uuid::Uuid;

    struct StubTicketMonitorPort {
        regions: Vec<TicketWatchRegion>,
    }

    #[async_trait]
    impl TicketMonitorPort for StubTicketMonitorPort {
        async fn fetch_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
            Ok(TicketWatchCurrentMatchView {
                current_match: None,
                group_ticket_active: false,
                message: String::new(),
            })
        }

        async fn fetch_all_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
            Ok(vec![])
        }

        async fn fetch_regions(&self) -> anyhow::Result<Vec<TicketWatchRegion>> {
            Ok(self.regions.clone())
        }

        async fn fetch_inventory(
            &self,
            _match_id: i64,
            _fallback_match_id: Option<i64>,
            _since: Option<&str>,
        ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>> {
            Ok(vec![])
        }

        async fn fetch_block_interests(
            &self,
            _match_id: i64,
            _viewer_user_id: Option<Uuid>,
        ) -> anyhow::Result<Vec<TicketWatchBlockInterest>> {
            Ok(vec![])
        }

        async fn fetch_tracked_interests(
            &self,
            _match_id: i64,
            _user_id: Uuid,
        ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>> {
            Ok(vec![])
        }

        async fn toggle_block_interest(
            &self,
            _match_id: i64,
            _user_id: Uuid,
            block_name: &str,
        ) -> anyhow::Result<TicketWatchBlockInterest> {
            Ok(TicketWatchBlockInterest {
                block_name: block_name.to_string(),
                interested_user_count: 0,
                viewer_interested: false,
            })
        }

        async fn fetch_yukun_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>> {
            Ok(vec![])
        }

        async fn fetch_yukun_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView> {
            self.fetch_current_match().await
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
    async fn execute_completes_phoenix_hill_region_template() {
        let use_case = ListTicketWatchRegionsUseCase::new(Arc::new(StubTicketMonitorPort {
            regions: vec![TicketWatchRegion {
                block_key: "101".to_string(),
                block_name: "101".to_string(),
                price: "280".to_string(),
                usable_count: 8,
                estate: 1,
            }],
        }));

        let regions = use_case.execute().await.unwrap();

        assert!(regions.iter().any(|region| region.block_key == "116"));
        assert!(regions.iter().any(|region| region.block_key == "536"));
        assert_eq!(
            regions
                .iter()
                .filter(|region| region.block_key == "101")
                .count(),
            1
        );
    }
}
