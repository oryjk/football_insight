use async_trait::async_trait;

use crate::seat_swap::domain::{SeatSwapCurrentMatch, SeatSwapRegion};

#[async_trait]
pub trait CurrentSeatSwapMatchPort: Send + Sync {
    async fn current_match(&self) -> anyhow::Result<Option<SeatSwapCurrentMatch>>;
    async fn current_regions(&self) -> anyhow::Result<Vec<SeatSwapRegion>>;
}
