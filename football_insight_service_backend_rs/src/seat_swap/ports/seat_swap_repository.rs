use async_trait::async_trait;
use uuid::Uuid;

use crate::seat_swap::domain::{SeatSwapDesiredSeat, SeatSwapRequest, SeatSwapRequestStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapConfirmation {
    pub request_id: Uuid,
    pub target_request_id: Uuid,
    pub confirmed_by_user_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertSeatSwapRequestInput {
    pub match_id: i64,
    pub user_id: Uuid,
    pub current_region_key: String,
    pub current_region_name: String,
    pub current_row: String,
    pub current_seat_no: String,
    pub wechat_id: Option<String>,
    pub phone_number: Option<String>,
    pub mini_program_notice_enabled: bool,
    pub desired_seats: Vec<SeatSwapDesiredSeat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchedCancellationInput {
    pub match_id: i64,
    pub request_id: Uuid,
    pub target_request_id: Uuid,
    pub cancelled_by_user_id: Uuid,
    pub reason: String,
    pub evidence_object_key: String,
    pub evidence_url: String,
}

#[async_trait]
pub trait SeatSwapRepository: Send + Sync {
    async fn list_active_requests(&self, match_id: i64) -> anyhow::Result<Vec<SeatSwapRequest>>;

    async fn find_request_by_user(
        &self,
        match_id: i64,
        user_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapRequest>>;

    async fn find_request_by_id(
        &self,
        request_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapRequest>> {
        let _ = request_id;
        Ok(None)
    }

    async fn find_confirmation(
        &self,
        match_id: i64,
        request_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapConfirmation>>;

    async fn list_confirmations_by_request(
        &self,
        match_id: i64,
        request_id: Uuid,
    ) -> anyhow::Result<Vec<SeatSwapConfirmation>> {
        Ok(self
            .find_confirmation(match_id, request_id)
            .await?
            .into_iter()
            .collect())
    }

    async fn find_confirmation_between(
        &self,
        match_id: i64,
        request_id: Uuid,
        target_request_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
        let confirmation = self.find_confirmation(match_id, request_id).await?;
        Ok(confirmation.filter(|item| item.target_request_id == target_request_id))
    }

    async fn upsert_request(
        &self,
        input: UpsertSeatSwapRequestInput,
    ) -> anyhow::Result<SeatSwapRequest> {
        let _ = input;
        anyhow::bail!("seat swap repository upsert is not implemented")
    }

    async fn cancel_request(
        &self,
        match_id: i64,
        user_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapRequest>> {
        let _ = (match_id, user_id);
        anyhow::bail!("seat swap repository cancel is not implemented")
    }

    async fn set_confirmation(
        &self,
        match_id: i64,
        request_id: Uuid,
        target_request_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<SeatSwapConfirmation> {
        let _ = (match_id, request_id, target_request_id, user_id);
        anyhow::bail!("seat swap repository confirmation is not implemented")
    }

    async fn delete_confirmation(
        &self,
        match_id: i64,
        request_id: Uuid,
        target_request_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<bool> {
        let _ = (match_id, request_id, target_request_id, user_id);
        anyhow::bail!("seat swap repository confirmation delete is not implemented")
    }

    async fn mark_matched(
        &self,
        request_id: Uuid,
        target_request_id: Uuid,
    ) -> anyhow::Result<bool> {
        let _ = (request_id, target_request_id);
        anyhow::bail!("seat swap repository match update is not implemented")
    }

    async fn update_status(
        &self,
        request_id: Uuid,
        status: SeatSwapRequestStatus,
    ) -> anyhow::Result<()> {
        let _ = (request_id, status);
        anyhow::bail!("seat swap repository status update is not implemented")
    }

    async fn insert_matched_cancellation(
        &self,
        input: MatchedCancellationInput,
    ) -> anyhow::Result<()> {
        let _ = input;
        anyhow::bail!("seat swap repository cancellation is not implemented")
    }
}
