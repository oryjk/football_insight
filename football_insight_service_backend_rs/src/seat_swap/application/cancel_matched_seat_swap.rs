use std::sync::Arc;

use uuid::Uuid;

use crate::seat_swap::{
    domain::SeatSwapError,
    ports::{
        current_match_port::CurrentSeatSwapMatchPort,
        evidence_storage_port::{SeatSwapEvidenceStoragePort, SeatSwapEvidenceUpload},
        seat_swap_repository::{MatchedCancellationInput, SeatSwapRepository},
    },
};

pub struct CancelMatchedSeatSwapUseCase {
    repository: Arc<dyn SeatSwapRepository>,
    current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
    evidence_storage: Arc<dyn SeatSwapEvidenceStoragePort>,
}

pub struct CancelMatchedSeatSwapInput {
    pub target_request_id: Uuid,
    pub reason: String,
    pub evidence: SeatSwapEvidenceUpload,
}

impl CancelMatchedSeatSwapUseCase {
    pub fn new(
        repository: Arc<dyn SeatSwapRepository>,
        current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
        evidence_storage: Arc<dyn SeatSwapEvidenceStoragePort>,
    ) -> Self {
        Self {
            repository,
            current_match_port,
            evidence_storage,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        input: CancelMatchedSeatSwapInput,
    ) -> Result<(), SeatSwapError> {
        let Some(current_match) = self.current_match_port.current_match().await? else {
            return Err(SeatSwapError::NoCurrentMatch);
        };
        let reason = input.reason.trim().to_string();
        if reason.is_empty() {
            return Err(SeatSwapError::CancelReasonRequired);
        }
        if input.evidence.bytes.is_empty() {
            return Err(SeatSwapError::CancelEvidenceRequired);
        }

        let mine = self
            .repository
            .find_request_by_user(current_match.match_id, user_id)
            .await?
            .ok_or(SeatSwapError::RequestNotFound)?;
        if mine.matched_request_id != Some(input.target_request_id) {
            return Err(SeatSwapError::CanOnlyCancelMatchedRequest);
        }
        let evidence = self
            .evidence_storage
            .upload_cancel_evidence(current_match.match_id, mine.id, input.evidence)
            .await?;
        self.repository
            .insert_matched_cancellation(MatchedCancellationInput {
                match_id: current_match.match_id,
                request_id: mine.id,
                target_request_id: input.target_request_id,
                cancelled_by_user_id: user_id,
                reason,
                evidence_object_key: evidence.object_key,
                evidence_url: evidence.public_url,
            })
            .await?;
        self.repository
            .update_status(
                mine.id,
                crate::seat_swap::domain::SeatSwapRequestStatus::Cancelled,
            )
            .await?;
        Ok(())
    }
}
