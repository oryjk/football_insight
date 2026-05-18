use std::sync::Arc;

use uuid::Uuid;

use crate::seat_swap::{
    domain::{SeatSwapContact, SeatSwapDesiredSeat, SeatSwapRequestStatus},
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
    ) -> anyhow::Result<crate::seat_swap::domain::SeatSwapRequest> {
        let Some(current_match) = self.current_match_port.current_match().await? else {
            anyhow::bail!("当前暂无可换座比赛");
        };

        let existing_request = self
            .repository
            .find_request_by_user(current_match.match_id, user_id)
            .await?;
        if existing_request
            .as_ref()
            .is_some_and(|request| request.status == SeatSwapRequestStatus::Matched)
        {
            anyhow::bail!("已正式匹配的换座不能更新，请先按撤销流程处理");
        }

        let regions = self.current_match_port.current_regions().await?;
        let is_valid_region =
            |region_key: &str| regions.iter().any(|region| region.region_key == region_key);

        if !is_valid_region(&input.current_region_key) {
            anyhow::bail!("请选择有效的当前分区");
        }

        let current_row = normalize_required(input.current_row, "请输入当前排号")?;
        let current_seat_no = normalize_required(input.current_seat_no, "请输入当前座号")?;
        if input.desired_seats.is_empty() {
            anyhow::bail!("请至少选择一个想换到的分区");
        }

        let desired_seats = input
            .desired_seats
            .into_iter()
            .map(|seat| {
                if !is_valid_region(&seat.region_key) {
                    anyhow::bail!("请选择有效的目标分区");
                }
                Ok(SeatSwapDesiredSeat {
                    region_key: seat.region_key,
                    region_name: seat.region_name,
                    desired_row: normalize_optional(seat.desired_row),
                    desired_seat_no: normalize_optional(seat.desired_seat_no),
                })
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        let contact = SeatSwapContact::new(input.wechat_id, input.phone_number)
            .map_err(|_| anyhow::anyhow!("请至少填写微信号或手机号"))?;

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
                desired_seats,
            })
            .await
    }
}

fn normalize_required(value: String, message: &str) -> anyhow::Result<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        anyhow::bail!(message.to_string());
    }
    Ok(value)
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
    async fn rejects_missing_current_match() {
        let error = use_case(None)
            .execute(USER_ID, valid_input())
            .await
            .expect_err("missing match should fail");

        assert!(error.to_string().contains("当前暂无可换座比赛"));
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

        assert!(error.to_string().contains("至少填写微信号或手机号"));
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

        assert!(error.to_string().contains("已正式匹配"));
    }
}
