use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapConfirmedNotification {
    pub recipient_user_id: Uuid,
    pub recipient_open_id: String,
    pub confirmer_display_name: String,
    pub current_region_name: String,
    pub current_row: String,
    pub current_seat_no: String,
    pub desired_region_summary: String,
}

#[async_trait]
pub trait SeatSwapMiniProgramSubscribePort: Send + Sync {
    async fn send_confirmed_notification(
        &self,
        payload: SeatSwapConfirmedNotification,
    ) -> anyhow::Result<()>;
}
