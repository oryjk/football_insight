use std::sync::Arc;

use uuid::Uuid;

use crate::{
    auth::ports::user_membership_port::UserMembershipPort,
    seat_swap::{
        domain::{SeatSwapError, SeatSwapRequest, SeatSwapRequestStatus},
        ports::{
            current_match_port::CurrentSeatSwapMatchPort,
            mini_program_subscribe_port::{
                SeatSwapConfirmedNotification, SeatSwapMiniProgramSubscribePort,
            },
            seat_swap_repository::{SeatSwapConfirmation, SeatSwapRepository},
        },
    },
};

pub struct ConfirmSeatSwapCandidateUseCase {
    repository: Arc<dyn SeatSwapRepository>,
    current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
    user_membership_port: Arc<dyn UserMembershipPort>,
    mini_subscribe_port: Arc<dyn SeatSwapMiniProgramSubscribePort>,
}

impl ConfirmSeatSwapCandidateUseCase {
    pub fn new(
        repository: Arc<dyn SeatSwapRepository>,
        current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
        user_membership_port: Arc<dyn UserMembershipPort>,
        mini_subscribe_port: Arc<dyn SeatSwapMiniProgramSubscribePort>,
    ) -> Self {
        Self {
            repository,
            current_match_port,
            user_membership_port,
            mini_subscribe_port,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        target_request_id: Uuid,
    ) -> Result<SeatSwapConfirmation, SeatSwapError> {
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
        ensure_request_can_confirm(&mine, true)?;
        ensure_request_can_confirm(&target, false)?;

        let confirmation = self
            .repository
            .set_confirmation(current_match.match_id, mine.id, target.id, user_id)
            .await?;
        let peer_confirmation = self
            .repository
            .find_confirmation_between(current_match.match_id, target.id, mine.id)
            .await?;

        if peer_confirmation
            .as_ref()
            .is_some_and(|item| item.target_request_id == mine.id)
        {
            if !self.repository.mark_matched(mine.id, target.id).await? {
                return Err(SeatSwapError::MatchAlreadyTaken);
            }
            self.notify_target_confirmed(&mine, &target).await?;
        }

        Ok(confirmation)
    }

    async fn notify_target_confirmed(
        &self,
        mine: &SeatSwapRequest,
        target: &SeatSwapRequest,
    ) -> anyhow::Result<()> {
        let Some(open_id) = self
            .user_membership_port
            .get_user_open_id(target.user.user_id)
            .await?
        else {
            return Ok(());
        };

        let desired_region_summary = mine
            .desired_seats
            .iter()
            .map(|seat| seat.region_name.clone())
            .collect::<Vec<_>>()
            .join(" / ");

        self.mini_subscribe_port
            .send_confirmed_notification(SeatSwapConfirmedNotification {
                recipient_user_id: target.user.user_id,
                recipient_open_id: open_id,
                confirmer_display_name: mine.user.display_name.clone(),
                current_region_name: mine.current_region_name.clone(),
                current_row: mine.current_row.clone(),
                current_seat_no: mine.current_seat_no.clone(),
                desired_region_summary,
            })
            .await
    }
}

fn ensure_request_can_confirm(
    request: &SeatSwapRequest,
    is_mine: bool,
) -> Result<(), SeatSwapError> {
    match request.status {
        SeatSwapRequestStatus::Active => Ok(()),
        SeatSwapRequestStatus::Matched if is_mine => {
            Err(SeatSwapError::MatchedRequestNeedsEvidenceCancellation)
        }
        SeatSwapRequestStatus::Matched => Err(SeatSwapError::TargetAlreadyMatched),
        SeatSwapRequestStatus::Cancelled => Err(SeatSwapError::TargetAlreadyCancelled),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::*;
    use crate::{
        auth::ports::user_membership_port::UserMembershipPort,
        seat_swap::{
            domain::{
                SeatSwapContact, SeatSwapCurrentMatch, SeatSwapDesiredSeat, SeatSwapRequest,
                SeatSwapRequestStatus, SeatSwapUser,
            },
            ports::{
                current_match_port::CurrentSeatSwapMatchPort,
                mini_program_subscribe_port::{
                    SeatSwapConfirmedNotification, SeatSwapMiniProgramSubscribePort,
                },
                seat_swap_repository::{SeatSwapConfirmation, SeatSwapRepository},
            },
        },
    };

    const USER_A: Uuid = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    const USER_B: Uuid = Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb);
    const USER_C: Uuid = Uuid::from_u128(0xcccccccccccccccccccccccccccccccc);
    const REQ_A: Uuid = Uuid::from_u128(1);
    const REQ_B: Uuid = Uuid::from_u128(2);
    const REQ_C: Uuid = Uuid::from_u128(3);

    struct StubRepository {
        peer_confirmed_me: bool,
        matched_calls: Mutex<Vec<(Uuid, Uuid)>>,
        requests: Vec<SeatSwapRequest>,
        confirmations: Mutex<Vec<SeatSwapConfirmation>>,
    }

    #[derive(Default)]
    struct StubUserMembershipPort;

    #[async_trait]
    impl UserMembershipPort for StubUserMembershipPort {
        async fn get_user_open_id(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            Ok(Some("openid-target".to_string()))
        }

        async fn get_user_membership_tier(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            Ok(None)
        }

        async fn update_user_membership_tier(
            &self,
            _user_id: Uuid,
            _tier: &str,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn is_seat_swap_notice_enabled(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<bool>> {
            Ok(Some(true))
        }
    }

    #[derive(Default)]
    struct StubMiniSubscribePort {
        sent: Mutex<Vec<SeatSwapConfirmedNotification>>,
    }

    #[async_trait]
    impl SeatSwapMiniProgramSubscribePort for StubMiniSubscribePort {
        async fn send_confirmed_notification(
            &self,
            payload: SeatSwapConfirmedNotification,
        ) -> anyhow::Result<()> {
            self.sent.lock().expect("sent").push(payload);
            Ok(())
        }
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
            Ok(self
                .requests
                .iter()
                .find(|request| request.user.user_id == user_id)
                .cloned())
        }

        async fn find_request_by_id(
            &self,
            request_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapRequest>> {
            Ok(self
                .requests
                .iter()
                .find(|request| request.id == request_id)
                .cloned())
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
            Ok(self
                .confirmations
                .lock()
                .expect("confirmations")
                .iter()
                .find(|confirmation| confirmation.request_id == request_id)
                .cloned())
        }

        async fn set_confirmation(
            &self,
            _match_id: i64,
            request_id: Uuid,
            target_request_id: Uuid,
            user_id: Uuid,
        ) -> anyhow::Result<SeatSwapConfirmation> {
            let confirmation = SeatSwapConfirmation {
                request_id,
                target_request_id,
                confirmed_by_user_id: user_id,
            };
            self.confirmations
                .lock()
                .expect("confirmations")
                .push(confirmation.clone());
            Ok(confirmation)
        }

        async fn mark_matched(
            &self,
            request_id: Uuid,
            target_request_id: Uuid,
        ) -> anyhow::Result<bool> {
            self.matched_calls
                .lock()
                .expect("calls")
                .push((request_id, target_request_id));
            Ok(true)
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
                avatar_url: None,
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
            seat_swap_notice_enabled: false,
            status: SeatSwapRequestStatus::Active,
            matched_request_id: None,
            created_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
        }
    }

    fn stub_repository(requests: Vec<SeatSwapRequest>) -> StubRepository {
        StubRepository {
            peer_confirmed_me: false,
            matched_calls: Mutex::new(Vec::new()),
            requests,
            confirmations: Mutex::new(Vec::new()),
        }
    }

    #[tokio::test]
    async fn marks_both_requests_matched_when_peer_already_confirmed_me() {
        let repository = Arc::new(StubRepository {
            peer_confirmed_me: true,
            matched_calls: Mutex::new(Vec::new()),
            requests: vec![
                request_fixture(REQ_A, USER_A, "A", vec!["B"]),
                request_fixture(REQ_B, USER_B, "B", vec!["A"]),
            ],
            confirmations: Mutex::new(Vec::new()),
        });
        let subscribe_port = Arc::new(StubMiniSubscribePort::default());
        let use_case = ConfirmSeatSwapCandidateUseCase::new(
            repository.clone(),
            Arc::new(StubCurrentMatchPort),
            Arc::new(StubUserMembershipPort),
            subscribe_port.clone(),
        );

        use_case.execute(USER_A, REQ_B).await.expect("confirm");

        assert_eq!(
            repository.matched_calls.lock().expect("calls").as_slice(),
            &[(REQ_A, REQ_B)]
        );
        assert_eq!(subscribe_port.sent.lock().expect("sent").len(), 1);
    }

    #[tokio::test]
    async fn allows_one_user_to_confirm_multiple_targets_before_final_match() {
        let repository = Arc::new(stub_repository(vec![
            request_fixture(REQ_A, USER_A, "A", vec!["B", "C"]),
            request_fixture(REQ_B, USER_B, "B", vec!["A"]),
            request_fixture(REQ_C, USER_C, "C", vec!["A"]),
        ]));
        let use_case = ConfirmSeatSwapCandidateUseCase::new(
            repository.clone(),
            Arc::new(StubCurrentMatchPort),
            Arc::new(StubUserMembershipPort),
            Arc::new(StubMiniSubscribePort::default()),
        );

        use_case.execute(USER_A, REQ_B).await.expect("confirm B");
        use_case.execute(USER_A, REQ_C).await.expect("confirm C");

        let confirmations = repository.confirmations.lock().expect("confirmations");
        assert_eq!(
            confirmations
                .iter()
                .map(|item| item.target_request_id)
                .collect::<Vec<_>>(),
            vec![REQ_B, REQ_C]
        );
        assert!(repository.matched_calls.lock().expect("calls").is_empty());
    }

    #[tokio::test]
    async fn rejects_confirm_after_my_request_is_already_matched() {
        let mut mine = request_fixture(REQ_A, USER_A, "A", vec!["B"]);
        mine.status = SeatSwapRequestStatus::Matched;
        mine.matched_request_id = Some(REQ_B);
        let repository = Arc::new(stub_repository(vec![
            mine,
            request_fixture(REQ_B, USER_B, "B", vec!["A"]),
        ]));
        let use_case = ConfirmSeatSwapCandidateUseCase::new(
            repository.clone(),
            Arc::new(StubCurrentMatchPort),
            Arc::new(StubUserMembershipPort),
            Arc::new(StubMiniSubscribePort::default()),
        );

        let error = use_case.execute(USER_A, REQ_B).await.expect_err("error");

        assert!(matches!(
            error,
            SeatSwapError::MatchedRequestNeedsEvidenceCancellation
        ));
        assert!(
            repository
                .confirmations
                .lock()
                .expect("confirmations")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn rejects_confirm_when_target_request_is_cancelled() {
        let mut target = request_fixture(REQ_B, USER_B, "B", vec!["A"]);
        target.status = SeatSwapRequestStatus::Cancelled;
        let repository = Arc::new(stub_repository(vec![
            request_fixture(REQ_A, USER_A, "A", vec!["B"]),
            target,
        ]));
        let use_case = ConfirmSeatSwapCandidateUseCase::new(
            repository.clone(),
            Arc::new(StubCurrentMatchPort),
            Arc::new(StubUserMembershipPort),
            Arc::new(StubMiniSubscribePort::default()),
        );

        let error = use_case.execute(USER_A, REQ_B).await.expect_err("error");

        assert!(matches!(error, SeatSwapError::TargetAlreadyCancelled));
        assert!(
            repository
                .confirmations
                .lock()
                .expect("confirmations")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn returns_business_error_when_final_match_was_taken_concurrently() {
        struct TakenMatchRepository;

        #[async_trait]
        impl SeatSwapRepository for TakenMatchRepository {
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
                Ok(Some(request_fixture(REQ_A, USER_A, "A", vec!["B"])))
            }

            async fn find_request_by_id(
                &self,
                _request_id: Uuid,
            ) -> anyhow::Result<Option<SeatSwapRequest>> {
                Ok(Some(request_fixture(REQ_B, USER_B, "B", vec!["A"])))
            }

            async fn find_confirmation(
                &self,
                _match_id: i64,
                _request_id: Uuid,
            ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
                Ok(None)
            }

            async fn find_confirmation_between(
                &self,
                _match_id: i64,
                request_id: Uuid,
                target_request_id: Uuid,
            ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
                Ok(Some(SeatSwapConfirmation {
                    request_id,
                    target_request_id,
                    confirmed_by_user_id: USER_B,
                }))
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
                _request_id: Uuid,
                _target_request_id: Uuid,
            ) -> anyhow::Result<bool> {
                Ok(false)
            }
        }

        let use_case = ConfirmSeatSwapCandidateUseCase::new(
            Arc::new(TakenMatchRepository),
            Arc::new(StubCurrentMatchPort),
            Arc::new(StubUserMembershipPort),
            Arc::new(StubMiniSubscribePort::default()),
        );

        let error = use_case.execute(USER_A, REQ_B).await.expect_err("error");

        assert!(matches!(error, SeatSwapError::MatchAlreadyTaken));
    }

    #[tokio::test]
    async fn allows_confirm_even_when_regions_are_not_bidirectionally_matched() {
        struct LooseStubRepository;

        #[async_trait]
        impl SeatSwapRepository for LooseStubRepository {
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
                Ok(Some(request_fixture(REQ_A, USER_A, "A", vec!["B"])))
            }

            async fn find_request_by_id(
                &self,
                _request_id: Uuid,
            ) -> anyhow::Result<Option<SeatSwapRequest>> {
                Ok(Some(request_fixture(REQ_B, USER_B, "C", vec!["D"])))
            }

            async fn find_confirmation(
                &self,
                _match_id: i64,
                _request_id: Uuid,
            ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
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
        }

        let use_case = ConfirmSeatSwapCandidateUseCase::new(
            Arc::new(LooseStubRepository),
            Arc::new(StubCurrentMatchPort),
            Arc::new(StubUserMembershipPort),
            Arc::new(StubMiniSubscribePort::default()),
        );

        let confirmation = use_case.execute(USER_A, REQ_B).await.expect("confirm");

        assert_eq!(confirmation.request_id, REQ_A);
        assert_eq!(confirmation.target_request_id, REQ_B);
    }
}
