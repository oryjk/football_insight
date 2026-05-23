use std::sync::Arc;

use uuid::Uuid;

use crate::seat_swap::{
    domain::{SeatSwapContact, SeatSwapDesiredSeat, SeatSwapError, SeatSwapRequestStatus},
    ports::{
        current_match_port::CurrentSeatSwapMatchPort,
        seat_swap_repository::{SeatSwapRepository, UpsertSeatSwapRequestInput},
    },
};

pub struct UpsertMySeatSwapRequestUseCase {
    repository: Arc<dyn SeatSwapRepository>,
    current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertMySeatSwapRequestInput {
    pub current_region_key: String,
    pub current_region_name: String,
    pub current_row: String,
    pub current_seat_no: String,
    pub wechat_id: Option<String>,
    pub phone_number: Option<String>,
    pub mini_program_notice_enabled: bool,
    pub desired_seats: Vec<UpsertDesiredSeatInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpsertDesiredSeatInput {
    pub region_key: String,
    pub region_name: String,
    pub desired_row: Option<String>,
    pub desired_seat_no: Option<String>,
}

impl UpsertMySeatSwapRequestUseCase {
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
        input: UpsertMySeatSwapRequestInput,
    ) -> Result<crate::seat_swap::domain::SeatSwapRequest, SeatSwapError> {
        let Some(current_match) = self.current_match_port.current_match().await? else {
            return Err(SeatSwapError::NoCurrentMatch);
        };

        let existing_request = self
            .repository
            .find_request_by_user(current_match.match_id, user_id)
            .await?;
        if existing_request
            .as_ref()
            .is_some_and(|request| request.status == SeatSwapRequestStatus::Matched)
        {
            return Err(SeatSwapError::MatchedRequestCannotUpdate);
        }

        let regions = self.current_match_port.current_regions().await?;
        let is_valid_region =
            |region_key: &str| regions.iter().any(|region| region.region_key == region_key);

        if !is_valid_region(&input.current_region_key) {
            return Err(SeatSwapError::InvalidCurrentRegion);
        }

        let current_row = normalize_to_string(input.current_row);
        let current_seat_no = normalize_to_string(input.current_seat_no);
        if current_row.is_empty() {
            return Err(SeatSwapError::CurrentRowRequired);
        }
        if current_seat_no.is_empty() {
            return Err(SeatSwapError::CurrentSeatNoRequired);
        }
        if input.desired_seats.is_empty() {
            return Err(SeatSwapError::DesiredSeatRequired);
        }

        let desired_seats = input
            .desired_seats
            .into_iter()
            .map(|seat| {
                if !is_valid_region(&seat.region_key) {
                    return Err(SeatSwapError::InvalidDesiredRegion);
                }
                Ok(SeatSwapDesiredSeat {
                    region_key: seat.region_key,
                    region_name: seat.region_name,
                    desired_row: normalize_optional(seat.desired_row),
                    desired_seat_no: normalize_optional(seat.desired_seat_no),
                })
            })
            .collect::<Result<Vec<_>, SeatSwapError>>()?;

        let contact = SeatSwapContact::new(input.wechat_id, input.phone_number).map_err(
            |error| match error {
                crate::seat_swap::domain::SeatSwapValidationError::InvalidPhoneNumber => {
                    SeatSwapError::InvalidPhoneNumber
                }
                _ => SeatSwapError::ContactRequired,
            },
        )?;

        self.repository
            .upsert_request(UpsertSeatSwapRequestInput {
                match_id: current_match.match_id,
                user_id,
                current_region_key: input.current_region_key,
                current_region_name: input.current_region_name,
                current_row,
                current_seat_no,
                wechat_id: contact.wechat_id,
                phone_number: contact.phone_number,
                mini_program_notice_enabled: input.mini_program_notice_enabled,
                desired_seats,
            })
            .await
            .map_err(SeatSwapError::from)
    }
}

fn normalize_to_string(value: String) -> String {
    value.trim().to_string()
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::*;
    use chrono::{TimeZone, Utc};

    use crate::seat_swap::{
        domain::{
            SeatSwapContact, SeatSwapCurrentMatch, SeatSwapRegion, SeatSwapRequest,
            SeatSwapRequestStatus, SeatSwapUser,
        },
        ports::{
            current_match_port::CurrentSeatSwapMatchPort,
            seat_swap_repository::{
                SeatSwapConfirmation, SeatSwapRepository, UpsertSeatSwapRequestInput,
            },
        },
    };

    const USER_ID: Uuid = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);

    #[derive(Default)]
    struct StubRepository {
        upserts: Mutex<Vec<UpsertSeatSwapRequestInput>>,
        existing_request: Mutex<Option<SeatSwapRequest>>,
    }

    #[async_trait]
    impl SeatSwapRepository for StubRepository {
        async fn list_active_requests(
            &self,
            _match_id: i64,
        ) -> anyhow::Result<Vec<crate::seat_swap::domain::SeatSwapRequest>> {
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

        async fn upsert_request(
            &self,
            input: UpsertSeatSwapRequestInput,
        ) -> anyhow::Result<crate::seat_swap::domain::SeatSwapRequest> {
            self.upserts.lock().expect("upserts").push(input.clone());
            Err(anyhow::anyhow!("stop after validation"))
        }
    }

    struct StubCurrentMatchPort {
        current_match: Option<SeatSwapCurrentMatch>,
        regions: Vec<SeatSwapRegion>,
    }

    #[async_trait]
    impl CurrentSeatSwapMatchPort for StubCurrentMatchPort {
        async fn current_match(&self) -> anyhow::Result<Option<SeatSwapCurrentMatch>> {
            Ok(self.current_match.clone())
        }

        async fn current_regions(&self) -> anyhow::Result<Vec<SeatSwapRegion>> {
            Ok(self.regions.clone())
        }
    }

    fn use_case(current_match: Option<SeatSwapCurrentMatch>) -> UpsertMySeatSwapRequestUseCase {
        use_case_with_repository(StubRepository::default(), current_match)
    }

    fn use_case_with_repository(
        repository: StubRepository,
        current_match: Option<SeatSwapCurrentMatch>,
    ) -> UpsertMySeatSwapRequestUseCase {
        UpsertMySeatSwapRequestUseCase::new(
            Arc::new(repository),
            Arc::new(StubCurrentMatchPort {
                current_match,
                regions: vec![SeatSwapRegion {
                    region_key: "A".to_string(),
                    region_name: "A区".to_string(),
                }],
            }),
        )
    }

    fn valid_input() -> UpsertMySeatSwapRequestInput {
        UpsertMySeatSwapRequestInput {
            current_region_key: "A".to_string(),
            current_region_name: "A区".to_string(),
            current_row: "8".to_string(),
            current_seat_no: "15".to_string(),
            wechat_id: Some("wx".to_string()),
            phone_number: None,
            mini_program_notice_enabled: false,
            desired_seats: vec![UpsertDesiredSeatInput {
                region_key: "A".to_string(),
                region_name: "A区".to_string(),
                desired_row: None,
                desired_seat_no: None,
            }],
        }
    }

    fn matched_request() -> SeatSwapRequest {
        SeatSwapRequest {
            id: Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb),
            match_id: 574,
            user: SeatSwapUser {
                user_id: USER_ID,
                display_name: "测试用户".to_string(),
                avatar_url: None,
            },
            current_region_key: "A".to_string(),
            current_region_name: "A区".to_string(),
            current_row: "8".to_string(),
            current_seat_no: "15".to_string(),
            desired_seats: vec![],
            contact: SeatSwapContact::new(Some("wx".to_string()), None).expect("contact"),
            seat_swap_notice_enabled: false,
            status: SeatSwapRequestStatus::Matched,
            matched_request_id: Some(Uuid::from_u128(0xcccccccccccccccccccccccccccccccc)),
            created_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
        }
    }

    #[tokio::test]
    async fn rejects_missing_current_match() {
        let error = use_case(None)
            .execute(USER_ID, valid_input())
            .await
            .expect_err("missing match should fail");

        assert!(matches!(error, SeatSwapError::NoCurrentMatch));
    }

    #[tokio::test]
    async fn rejects_missing_contact() {
        let mut input = valid_input();
        input.wechat_id = None;
        input.phone_number = None;

        let error = use_case(Some(SeatSwapCurrentMatch {
            match_id: 574,
            home_team_name: "成都蓉城".to_string(),
            away_team_name: "上海申花".to_string(),
            kickoff_at: "2026-05-18T19:35:00+08:00".to_string(),
        }))
        .execute(USER_ID, input)
        .await
        .expect_err("contact should fail");

        assert!(matches!(error, SeatSwapError::ContactRequired));
    }

    #[tokio::test]
    async fn rejects_missing_current_row() {
        let repository = StubRepository::default();
        let use_case = use_case_with_repository(
            repository,
            Some(SeatSwapCurrentMatch {
                match_id: 574,
                home_team_name: "成都蓉城".to_string(),
                away_team_name: "上海申花".to_string(),
                kickoff_at: "2026-05-18T19:35:00+08:00".to_string(),
            }),
        );
        let mut input = valid_input();
        input.current_row = "   ".to_string();

        let error = use_case
            .execute(USER_ID, input)
            .await
            .expect_err("missing current row should fail");

        assert!(matches!(error, SeatSwapError::CurrentRowRequired));
    }

    #[tokio::test]
    async fn rejects_missing_current_seat_no() {
        let repository = StubRepository::default();
        let use_case = use_case_with_repository(
            repository,
            Some(SeatSwapCurrentMatch {
                match_id: 574,
                home_team_name: "成都蓉城".to_string(),
                away_team_name: "上海申花".to_string(),
                kickoff_at: "2026-05-18T19:35:00+08:00".to_string(),
            }),
        );
        let mut input = valid_input();
        input.current_seat_no = "".to_string();

        let error = use_case
            .execute(USER_ID, input)
            .await
            .expect_err("missing current seat no should fail");

        assert!(matches!(error, SeatSwapError::CurrentSeatNoRequired));
    }

    #[tokio::test]
    async fn rejects_update_after_request_is_matched() {
        let repository = StubRepository {
            upserts: Mutex::new(Vec::new()),
            existing_request: Mutex::new(Some(matched_request())),
        };
        let error = use_case_with_repository(
            repository,
            Some(SeatSwapCurrentMatch {
                match_id: 574,
                home_team_name: "成都蓉城".to_string(),
                away_team_name: "上海申花".to_string(),
                kickoff_at: "2026-05-18T19:35:00+08:00".to_string(),
            }),
        )
        .execute(USER_ID, valid_input())
        .await
        .expect_err("matched request should not be updated");

        assert!(matches!(error, SeatSwapError::MatchedRequestCannotUpdate));
    }
}
