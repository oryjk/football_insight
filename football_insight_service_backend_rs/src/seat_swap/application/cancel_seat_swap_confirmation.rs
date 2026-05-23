use std::sync::Arc;

use uuid::Uuid;

use crate::seat_swap::{
    domain::{SeatSwapError, SeatSwapRequestStatus},
    ports::{
        current_match_port::CurrentSeatSwapMatchPort, seat_swap_repository::SeatSwapRepository,
    },
};

pub struct CancelSeatSwapConfirmationUseCase {
    repository: Arc<dyn SeatSwapRepository>,
    current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
}

impl CancelSeatSwapConfirmationUseCase {
    pub fn new(
        repository: Arc<dyn SeatSwapRepository>,
        current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
    ) -> Self {
        Self {
            repository,
            current_match_port,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        target_request_id: Uuid,
    ) -> Result<(), SeatSwapError> {
        let Some(current_match) = self.current_match_port.current_match().await? else {
            return Err(SeatSwapError::NoCurrentMatch);
        };
        let mine = self
            .repository
            .find_request_by_user(current_match.match_id, user_id)
            .await?
            .ok_or(SeatSwapError::MyRequestRequired)?;
        let target = self
            .repository
            .find_request_by_id(target_request_id)
            .await?
            .ok_or(SeatSwapError::TargetRequestNotFound)?;

        if mine.match_id != current_match.match_id || target.match_id != current_match.match_id {
            return Err(SeatSwapError::ConfirmTargetNotCurrentMatch);
        }
        if mine.status == SeatSwapRequestStatus::Matched {
            return Err(SeatSwapError::MatchedRequestNeedsEvidenceCancellation);
        }

        let deleted = self
            .repository
            .delete_confirmation(current_match.match_id, mine.id, target.id, user_id)
            .await?;
        if !deleted {
            return Err(SeatSwapError::ConfirmationNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::*;
    use crate::seat_swap::{
        domain::{
            SeatSwapContact, SeatSwapCurrentMatch, SeatSwapDesiredSeat, SeatSwapRequest,
            SeatSwapUser,
        },
        ports::{
            current_match_port::CurrentSeatSwapMatchPort,
            seat_swap_repository::{SeatSwapConfirmation, SeatSwapRepository},
        },
    };

    const USER_A: Uuid = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    const REQ_A: Uuid = Uuid::from_u128(1);
    const REQ_B: Uuid = Uuid::from_u128(2);

    struct StubRepository {
        mine_status: SeatSwapRequestStatus,
        delete_result: bool,
        delete_calls: Mutex<Vec<(i64, Uuid, Uuid, Uuid)>>,
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
            user_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapRequest>> {
            Ok(Some(request_fixture(
                REQ_A,
                user_id,
                self.mine_status.clone(),
            )))
        }

        async fn find_request_by_id(
            &self,
            request_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapRequest>> {
            Ok(Some(request_fixture(
                request_id,
                Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb),
                SeatSwapRequestStatus::Active,
            )))
        }

        async fn find_confirmation(
            &self,
            _match_id: i64,
            _request_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
            Ok(None)
        }

        async fn delete_confirmation(
            &self,
            match_id: i64,
            request_id: Uuid,
            target_request_id: Uuid,
            user_id: Uuid,
        ) -> anyhow::Result<bool> {
            self.delete_calls.lock().expect("delete calls").push((
                match_id,
                request_id,
                target_request_id,
                user_id,
            ));
            Ok(self.delete_result)
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
                kickoff_at: "2026-05-18T19:35:00+08:00".to_string(),
            }))
        }

        async fn current_regions(
            &self,
        ) -> anyhow::Result<Vec<crate::seat_swap::domain::SeatSwapRegion>> {
            Ok(vec![])
        }
    }

    fn request_fixture(id: Uuid, user_id: Uuid, status: SeatSwapRequestStatus) -> SeatSwapRequest {
        SeatSwapRequest {
            id,
            match_id: 574,
            user: SeatSwapUser {
                user_id,
                display_name: "球迷".to_string(),
                avatar_url: None,
            },
            current_region_key: "101".to_string(),
            current_region_name: "101".to_string(),
            current_row: "8".to_string(),
            current_seat_no: "15".to_string(),
            desired_seats: vec![SeatSwapDesiredSeat {
                region_key: "102".to_string(),
                region_name: "102".to_string(),
                desired_row: None,
                desired_seat_no: None,
            }],
            contact: SeatSwapContact::new(Some("wx".to_string()), None).expect("contact"),
            seat_swap_notice_enabled: false,
            status,
            matched_request_id: None,
            created_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
        }
    }

    #[tokio::test]
    async fn deletes_my_pending_confirmation_for_target_request() {
        let repository = Arc::new(StubRepository {
            mine_status: SeatSwapRequestStatus::Active,
            delete_result: true,
            delete_calls: Mutex::new(Vec::new()),
        });
        let use_case = CancelSeatSwapConfirmationUseCase::new(
            repository.clone(),
            Arc::new(StubCurrentMatchPort),
        );

        use_case.execute(USER_A, REQ_B).await.expect("cancel");

        assert_eq!(
            repository
                .delete_calls
                .lock()
                .expect("delete calls")
                .as_slice(),
            &[(574, REQ_A, REQ_B, USER_A)]
        );
    }

    #[tokio::test]
    async fn rejects_matched_request_so_formal_cancel_flow_is_used() {
        let repository = Arc::new(StubRepository {
            mine_status: SeatSwapRequestStatus::Matched,
            delete_result: true,
            delete_calls: Mutex::new(Vec::new()),
        });
        let use_case = CancelSeatSwapConfirmationUseCase::new(
            repository.clone(),
            Arc::new(StubCurrentMatchPort),
        );

        let error = use_case.execute(USER_A, REQ_B).await.expect_err("error");

        assert!(matches!(
            error,
            SeatSwapError::MatchedRequestNeedsEvidenceCancellation
        ));
        assert!(
            repository
                .delete_calls
                .lock()
                .expect("delete calls")
                .is_empty()
        );
    }
}
