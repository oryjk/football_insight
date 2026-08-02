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
        let mine = self
            .repository
            .find_request_by_user(current_match.match_id, user_id)
            .await?
            .ok_or(SeatSwapError::RequestNotFound)?;
        if mine.matched_request_id != Some(input.target_request_id) {
            return Err(SeatSwapError::CanOnlyCancelMatchedRequest);
        }
        let evidence = if input.evidence.bytes.is_empty() {
            None
        } else {
            Some(
                self.evidence_storage
                    .upload_cancel_evidence(current_match.match_id, mine.id, input.evidence)
                    .await?,
            )
        };
        self.repository
            .insert_matched_cancellation(MatchedCancellationInput {
                match_id: current_match.match_id,
                request_id: mine.id,
                target_request_id: input.target_request_id,
                cancelled_by_user_id: user_id,
                reason,
                evidence_object_key: evidence
                    .as_ref()
                    .map(|item| item.object_key.clone())
                    .unwrap_or_default(),
                evidence_url: evidence
                    .as_ref()
                    .map(|item| item.public_url.clone())
                    .unwrap_or_default(),
            })
            .await?;
        self.repository
            .cancel_matched_pair(mine.id, input.target_request_id)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{CancelMatchedSeatSwapInput, CancelMatchedSeatSwapUseCase};
    use crate::seat_swap::{
        domain::{
            SeatSwapContact, SeatSwapCurrentMatch, SeatSwapRequest, SeatSwapRequestStatus,
            SeatSwapUser,
        },
        ports::{
            current_match_port::CurrentSeatSwapMatchPort,
            evidence_storage_port::{
                SeatSwapEvidenceObject, SeatSwapEvidenceStoragePort, SeatSwapEvidenceUpload,
            },
            seat_swap_repository::{
                MatchedCancellationInput, SeatSwapConfirmation, SeatSwapRepository,
            },
        },
    };

    const USER_ID: Uuid = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    const REQUEST_ID: Uuid = Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb);
    const TARGET_ID: Uuid = Uuid::from_u128(0xcccccccccccccccccccccccccccccccc);

    struct StubRepository {
        cancellations: Mutex<Vec<MatchedCancellationInput>>,
        cancelled_pairs: Mutex<Vec<(Uuid, Uuid)>>,
    }

    #[async_trait]
    impl SeatSwapRepository for StubRepository {
        async fn list_active_requests(
            &self,
            _match_id: i64,
        ) -> anyhow::Result<Vec<SeatSwapRequest>> {
            Ok(vec![])
        }

        async fn find_request_by_user(
            &self,
            _match_id: i64,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapRequest>> {
            Ok(Some(SeatSwapRequest {
                id: REQUEST_ID,
                match_id: 574,
                user: SeatSwapUser {
                    user_id: USER_ID,
                    display_name: "测试用户".to_string(),
                    avatar_url: None,
                },
                current_region_key: "522".to_string(),
                current_region_name: "522".to_string(),
                current_row: "10".to_string(),
                current_seat_no: "20".to_string(),
                desired_seats: vec![],
                contact: SeatSwapContact::new(Some("wx".to_string()), None).expect("contact"),
                seat_swap_notice_enabled: false,
                status: SeatSwapRequestStatus::Matched,
                matched_request_id: Some(TARGET_ID),
                created_at: Utc.with_ymd_and_hms(2026, 5, 29, 12, 0, 0).unwrap(),
                updated_at: Utc.with_ymd_and_hms(2026, 5, 29, 12, 0, 0).unwrap(),
            }))
        }

        async fn find_confirmation(
            &self,
            _match_id: i64,
            _request_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
            Ok(None)
        }

        async fn insert_matched_cancellation(
            &self,
            input: MatchedCancellationInput,
        ) -> anyhow::Result<()> {
            self.cancellations
                .lock()
                .expect("cancellations")
                .push(input);
            Ok(())
        }

        async fn cancel_matched_pair(
            &self,
            request_id: Uuid,
            target_request_id: Uuid,
        ) -> anyhow::Result<()> {
            self.cancelled_pairs
                .lock()
                .expect("cancelled pairs")
                .push((request_id, target_request_id));
            Ok(())
        }
    }

    struct StubCurrentMatchPort;

    #[async_trait]
    impl CurrentSeatSwapMatchPort for StubCurrentMatchPort {
        async fn current_match(&self) -> anyhow::Result<Option<SeatSwapCurrentMatch>> {
            Ok(Some(SeatSwapCurrentMatch {
                match_id: 574,
                home_team_name: "成都蓉城".to_string(),
                away_team_name: "上海申花".to_string(),
                kickoff_at: "2026-05-29T19:35:00+08:00".to_string(),
            }))
        }

        async fn current_regions(
            &self,
        ) -> anyhow::Result<Vec<crate::seat_swap::domain::SeatSwapRegion>> {
            Ok(vec![])
        }
    }

    struct StubEvidenceStorage {
        upload_calls: Mutex<usize>,
    }

    #[async_trait]
    impl SeatSwapEvidenceStoragePort for StubEvidenceStorage {
        async fn upload_cancel_evidence(
            &self,
            _match_id: i64,
            _request_id: Uuid,
            _upload: SeatSwapEvidenceUpload,
        ) -> anyhow::Result<SeatSwapEvidenceObject> {
            *self.upload_calls.lock().expect("upload calls") += 1;
            Ok(SeatSwapEvidenceObject {
                object_key: "evidence/mock.png".to_string(),
                public_url: "https://example.com/evidence/mock.png".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn allows_cancelling_a_matched_request_without_uploading_evidence() {
        let repository = Arc::new(StubRepository {
            cancellations: Mutex::new(Vec::new()),
            cancelled_pairs: Mutex::new(Vec::new()),
        });
        let evidence_storage = Arc::new(StubEvidenceStorage {
            upload_calls: Mutex::new(0),
        });
        let use_case = CancelMatchedSeatSwapUseCase::new(
            repository.clone(),
            Arc::new(StubCurrentMatchPort),
            evidence_storage.clone(),
        );

        let result = use_case
            .execute(
                USER_ID,
                CancelMatchedSeatSwapInput {
                    target_request_id: TARGET_ID,
                    reason: "双方协商取消".to_string(),
                    evidence: SeatSwapEvidenceUpload {
                        file_name: "".to_string(),
                        content_type: "".to_string(),
                        bytes: vec![],
                    },
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(
            *evidence_storage.upload_calls.lock().expect("upload calls"),
            0
        );
        assert_eq!(
            repository
                .cancellations
                .lock()
                .expect("cancellations")
                .len(),
            1
        );
        assert_eq!(
            repository
                .cancelled_pairs
                .lock()
                .expect("cancelled pairs")
                .as_slice(),
            &[(REQUEST_ID, TARGET_ID)],
        );
    }
}
