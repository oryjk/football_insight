use std::sync::Arc;

use uuid::Uuid;

use crate::seat_swap::{
    domain::SeatSwapRequestStatus,
    ports::{
        current_match_port::CurrentSeatSwapMatchPort, seat_swap_repository::SeatSwapRepository,
    },
};

pub struct CancelMySeatSwapRequestUseCase {
    repository: Arc<dyn SeatSwapRepository>,
    current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
}

impl CancelMySeatSwapRequestUseCase {
    pub fn new(
        repository: Arc<dyn SeatSwapRepository>,
        current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
    ) -> Self {
        Self {
            repository,
            current_match_port,
        }
    }

    pub async fn execute(&self, user_id: Uuid) -> anyhow::Result<()> {
        let Some(current_match) = self.current_match_port.current_match().await? else {
            anyhow::bail!("当前暂无可换座比赛");
        };
        let request = self
            .repository
            .find_request_by_user(current_match.match_id, user_id)
            .await?;

        if request
            .as_ref()
            .is_some_and(|request| request.status == SeatSwapRequestStatus::Matched)
        {
            anyhow::bail!("已正式匹配的换座需要提交撤销说明和截图");
        }

        self.repository
            .cancel_request(current_match.match_id, user_id)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::seat_swap::{
        domain::{SeatSwapContact, SeatSwapCurrentMatch, SeatSwapRequest, SeatSwapUser},
        ports::{
            current_match_port::CurrentSeatSwapMatchPort,
            seat_swap_repository::{SeatSwapConfirmation, SeatSwapRepository},
        },
    };

    const USER_ID: Uuid = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);

    struct StubRepository {
        existing_request: Mutex<Option<SeatSwapRequest>>,
        cancel_calls: Mutex<usize>,
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
            Ok(self
                .existing_request
                .lock()
                .expect("existing request")
                .clone())
        }

        async fn find_confirmation(
            &self,
            _match_id: i64,
            _request_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
            Ok(None)
        }

        async fn cancel_request(
            &self,
            _match_id: i64,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapRequest>> {
            *self.cancel_calls.lock().expect("cancel calls") += 1;
            Ok(None)
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

    fn matched_request() -> SeatSwapRequest {
        SeatSwapRequest {
            id: Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb),
            match_id: 574,
            user: SeatSwapUser {
                user_id: USER_ID,
                display_name: "测试用户".to_string(),
            },
            current_region_key: "A".to_string(),
            current_region_name: "A区".to_string(),
            current_row: "8".to_string(),
            current_seat_no: "15".to_string(),
            desired_seats: vec![],
            contact: SeatSwapContact::new(Some("wx".to_string()), None).expect("contact"),
            status: SeatSwapRequestStatus::Matched,
            matched_request_id: Some(Uuid::from_u128(0xcccccccccccccccccccccccccccccccc)),
            created_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
        }
    }

    #[tokio::test]
    async fn rejects_plain_cancel_after_request_is_matched() {
        let repository = Arc::new(StubRepository {
            existing_request: Mutex::new(Some(matched_request())),
            cancel_calls: Mutex::new(0),
        });
        let use_case =
            CancelMySeatSwapRequestUseCase::new(repository.clone(), Arc::new(StubCurrentMatchPort));

        let error = use_case
            .execute(USER_ID)
            .await
            .expect_err("matched request should require evidence cancellation");

        assert!(error.to_string().contains("撤销说明和截图"));
        assert_eq!(*repository.cancel_calls.lock().expect("cancel calls"), 0);
    }
}
