use std::sync::Arc;

use uuid::Uuid;

use crate::seat_swap::ports::{
    current_match_port::CurrentSeatSwapMatchPort,
    seat_swap_repository::{SeatSwapConfirmation, SeatSwapRepository},
};

pub struct ConfirmSeatSwapCandidateUseCase {
    repository: Arc<dyn SeatSwapRepository>,
    current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
}

impl ConfirmSeatSwapCandidateUseCase {
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
    ) -> anyhow::Result<SeatSwapConfirmation> {
        let Some(current_match) = self.current_match_port.current_match().await? else {
            anyhow::bail!("当前暂无可换座比赛");
        };
        let mine = self
            .repository
            .find_request_by_user(current_match.match_id, user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("请先发布我的换座请求"))?;
        let target = self
            .repository
            .find_request_by_id(target_request_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("换座对象不存在"))?;

        if mine.match_id != current_match.match_id || target.match_id != current_match.match_id {
            anyhow::bail!("只能确认当前比赛的换座对象");
        }

        if !mine.has_bidirectional_match_with(&target) {
            anyhow::bail!("只能确认双向匹配的换座对象");
        }

        let confirmation = self
            .repository
            .set_confirmation(current_match.match_id, mine.id, target.id, user_id)
            .await?;
        let peer_confirmation = self
            .repository
            .find_confirmation(current_match.match_id, target.id)
            .await?;

        if peer_confirmation
            .as_ref()
            .is_some_and(|item| item.target_request_id == mine.id)
        {
            self.repository.mark_matched(mine.id, target.id).await?;
        }

        Ok(confirmation)
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
            SeatSwapRequestStatus, SeatSwapUser,
        },
        ports::{
            current_match_port::CurrentSeatSwapMatchPort,
            seat_swap_repository::{SeatSwapConfirmation, SeatSwapRepository},
        },
    };

    const USER_A: Uuid = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    const USER_B: Uuid = Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb);
    const REQ_A: Uuid = Uuid::from_u128(1);
    const REQ_B: Uuid = Uuid::from_u128(2);

    struct StubRepository {
        peer_confirmed_me: bool,
        matched_calls: Mutex<Vec<(Uuid, Uuid)>>,
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
            Ok(Some(if user_id == USER_A {
                request_fixture(REQ_A, USER_A, "A", vec!["B"])
            } else {
                request_fixture(REQ_B, USER_B, "B", vec!["A"])
            }))
        }

        async fn find_request_by_id(
            &self,
            request_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapRequest>> {
            Ok(Some(if request_id == REQ_B {
                request_fixture(REQ_B, USER_B, "B", vec!["A"])
            } else {
                request_fixture(REQ_A, USER_A, "A", vec!["B"])
            }))
        }

        async fn find_confirmation(
            &self,
            _match_id: i64,
            request_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
            if self.peer_confirmed_me && request_id == REQ_B {
                return Ok(Some(SeatSwapConfirmation {
                    request_id: REQ_B,
                    target_request_id: REQ_A,
                    confirmed_by_user_id: USER_B,
                }));
            }
            Ok(None)
        }

        async fn set_confirmation(
            &self,
            _match_id: i64,
            request_id: Uuid,
            target_request_id: Uuid,
            user_id: Uuid,
        ) -> anyhow::Result<SeatSwapConfirmation> {
            Ok(SeatSwapConfirmation {
                request_id,
                target_request_id,
                confirmed_by_user_id: user_id,
            })
        }

        async fn mark_matched(
            &self,
            request_id: Uuid,
            target_request_id: Uuid,
        ) -> anyhow::Result<()> {
            self.matched_calls
                .lock()
                .expect("calls")
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
                kickoff_at: "2026-05-18T19:35:00+08:00".to_string(),
            }))
        }

        async fn current_regions(
            &self,
        ) -> anyhow::Result<Vec<crate::seat_swap::domain::SeatSwapRegion>> {
            Ok(vec![])
        }
    }

    fn request_fixture(
        id: Uuid,
        user_id: Uuid,
        current_region_key: &str,
        desired: Vec<&str>,
    ) -> SeatSwapRequest {
        SeatSwapRequest {
            id,
            match_id: 574,
            user: SeatSwapUser {
                user_id,
                display_name: "球迷".to_string(),
            },
            current_region_key: current_region_key.to_string(),
            current_region_name: format!("{current_region_key}区"),
            current_row: "8".to_string(),
            current_seat_no: "15".to_string(),
            desired_seats: desired
                .into_iter()
                .map(|region_key| SeatSwapDesiredSeat {
                    region_key: region_key.to_string(),
                    region_name: format!("{region_key}区"),
                    desired_row: None,
                    desired_seat_no: None,
                })
                .collect(),
            contact: SeatSwapContact::new(Some("wx".to_string()), None).expect("contact"),
            status: SeatSwapRequestStatus::Active,
            matched_request_id: None,
            created_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
        }
    }

    #[tokio::test]
    async fn marks_both_requests_matched_when_peer_already_confirmed_me() {
        let repository = Arc::new(StubRepository {
            peer_confirmed_me: true,
            matched_calls: Mutex::new(Vec::new()),
        });
        let use_case = ConfirmSeatSwapCandidateUseCase::new(
            repository.clone(),
            Arc::new(StubCurrentMatchPort),
        );

        use_case.execute(USER_A, REQ_B).await.expect("confirm");

        assert_eq!(
            repository.matched_calls.lock().expect("calls").as_slice(),
            &[(REQ_A, REQ_B)]
        );
    }
}
