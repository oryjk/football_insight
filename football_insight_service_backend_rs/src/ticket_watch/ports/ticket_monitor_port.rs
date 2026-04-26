use async_trait::async_trait;

use crate::ticket_watch::domain::ticket_watch::{
    TicketWatchBlockInterest, TicketWatchCurrentMatchView, TicketWatchInventoryEntry,
    TicketWatchMatchSummary, TicketWatchRegion, TicketWatchTrackedInterest,
};
use uuid::Uuid;

#[async_trait]
pub trait TicketMonitorPort: Send + Sync {
    async fn fetch_current_match(&self) -> anyhow::Result<TicketWatchCurrentMatchView>;
    async fn fetch_all_matches(&self) -> anyhow::Result<Vec<TicketWatchMatchSummary>>;
    async fn fetch_regions(&self) -> anyhow::Result<Vec<TicketWatchRegion>>;
    async fn fetch_inventory(
        &self,
        match_id: i64,
        fallback_match_id: Option<i64>,
        since: Option<&str>,
    ) -> anyhow::Result<Vec<TicketWatchInventoryEntry>>;

    async fn fetch_block_interests(
        &self,
        match_id: i64,
        viewer_user_id: Option<Uuid>,
    ) -> anyhow::Result<Vec<TicketWatchBlockInterest>>;

    async fn fetch_tracked_interests(
        &self,
        match_id: i64,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<TicketWatchTrackedInterest>>;

    async fn toggle_block_interest(
        &self,
        match_id: i64,
        user_id: Uuid,
        block_name: &str,
    ) -> anyhow::Result<TicketWatchBlockInterest>;
}
