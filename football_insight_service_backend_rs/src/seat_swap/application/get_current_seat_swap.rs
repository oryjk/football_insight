use std::sync::Arc;

use uuid::Uuid;

use crate::seat_swap::{
    domain::{
        SeatSwapCandidateStatus, SeatSwapContact, SeatSwapCurrentMatch, SeatSwapDesiredSeat,
        SeatSwapRequest,
    },
    ports::{
        current_match_port::CurrentSeatSwapMatchPort,
        seat_swap_repository::{SeatSwapConfirmation, SeatSwapRepository},
    },
};

pub struct GetCurrentSeatSwapUseCase {
    repository: Arc<dyn SeatSwapRepository>,
    current_match_port: Arc<dyn CurrentSeatSwapMatchPort>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapCurrentView {
    pub available: bool,
    pub current_match: Option<SeatSwapCurrentMatch>,
    pub my_request: Option<SeatSwapRequestView>,
    pub candidates: Vec<SeatSwapCandidateView>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapRequestView {
    pub request_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub current_region_key: String,
    pub current_region_name: String,
    pub current_row: String,
    pub current_seat_no: String,
    pub desired_seats: Vec<SeatSwapDesiredSeat>,
    pub contact: Option<SeatSwapContact>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapCandidateView {
    pub request_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub current_region_key: String,
    pub current_region_name: String,
    pub current_row: String,
    pub current_seat_no: String,
    pub desired_seats: Vec<SeatSwapDesiredSeat>,
    pub contact: Option<SeatSwapContact>,
    pub status: SeatSwapCandidateStatus,
    pub created_at: String,
}

impl GetCurrentSeatSwapUseCase {
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
        viewer_user_id: Option<Uuid>,
    ) -> anyhow::Result<SeatSwapCurrentView> {
        let Some(current_match) = self.current_match_port.current_match().await? else {
            return Ok(SeatSwapCurrentView {
                available: false,
                current_match: None,
                my_request: None,
                candidates: Vec::new(),
            });
        };

        let requests = self
            .repository
            .list_active_requests(current_match.match_id)
            .await?;
        let my_request = viewer_user_id
            .and_then(|user_id| {
                requests
                    .iter()
                    .find(|request| request.user.user_id == user_id)
            })
            .cloned();
        let my_confirmation = match my_request.as_ref() {
            Some(request) => {
                self.repository
                    .find_confirmation(current_match.match_id, request.id)
                    .await?
            }
            None => None,
        };

        let mut candidates = Vec::new();
        for request in requests.iter() {
            if my_request
                .as_ref()
                .is_some_and(|mine| mine.id == request.id)
            {
                continue;
            }

            let peer_confirmation = self
                .repository
                .find_confirmation(current_match.match_id, request.id)
                .await?;
            let status = resolve_candidate_status(
                my_request.as_ref(),
                request,
                my_confirmation.as_ref(),
                peer_confirmation.as_ref(),
            );
            let show_contact = should_show_contact(viewer_user_id, &status);
            candidates.push(to_candidate_view(request, status, show_contact));
        }

        Ok(SeatSwapCurrentView {
            available: true,
            current_match: Some(current_match),
            my_request: my_request
                .as_ref()
                .map(|request| to_request_view(request, true)),
            candidates,
        })
    }
}

fn resolve_candidate_status(
    my_request: Option<&SeatSwapRequest>,
    peer: &SeatSwapRequest,
    my_confirmation: Option<&SeatSwapConfirmation>,
    peer_confirmation: Option<&SeatSwapConfirmation>,
) -> SeatSwapCandidateStatus {
    let Some(mine) = my_request else {
        return SeatSwapCandidateStatus::DisplayOnly;
    };

    if mine.matched_request_id == Some(peer.id) && peer.matched_request_id == Some(mine.id) {
        return SeatSwapCandidateStatus::Matched;
    }

    if !mine.has_bidirectional_match_with(peer) {
        return SeatSwapCandidateStatus::DisplayOnly;
    }

    let i_confirmed_peer = my_confirmation.is_some_and(|item| item.target_request_id == peer.id);
    let peer_confirmed_me = peer_confirmation.is_some_and(|item| item.target_request_id == mine.id);

    match (i_confirmed_peer, peer_confirmed_me) {
        (true, true) => SeatSwapCandidateStatus::Matched,
        (true, false) => SeatSwapCandidateStatus::WaitingPeerConfirmation,
        (false, true) => SeatSwapCandidateStatus::PeerConfirmedMe,
        (false, false) => SeatSwapCandidateStatus::Communicable,
    }
}

fn should_show_contact(viewer_user_id: Option<Uuid>, status: &SeatSwapCandidateStatus) -> bool {
    viewer_user_id.is_some()
        && matches!(
            status,
            SeatSwapCandidateStatus::Communicable
                | SeatSwapCandidateStatus::WaitingPeerConfirmation
                | SeatSwapCandidateStatus::PeerConfirmedMe
                | SeatSwapCandidateStatus::Matched
        )
}

fn to_request_view(request: &SeatSwapRequest, show_contact: bool) -> SeatSwapRequestView {
    SeatSwapRequestView {
        request_id: request.id,
        user_id: request.user.user_id,
        display_name: request.user.display_name.clone(),
        current_region_key: request.current_region_key.clone(),
        current_region_name: request.current_region_name.clone(),
        current_row: request.current_row.clone(),
        current_seat_no: request.current_seat_no.clone(),
        desired_seats: request.desired_seats.clone(),
        contact: show_contact.then(|| request.contact.clone()),
        status: request.status.as_str().to_string(),
        created_at: request.created_at.to_rfc3339(),
    }
}

fn to_candidate_view(
    request: &SeatSwapRequest,
    status: SeatSwapCandidateStatus,
    show_contact: bool,
) -> SeatSwapCandidateView {
    SeatSwapCandidateView {
        request_id: request.id,
        user_id: request.user.user_id,
        display_name: request.user.display_name.clone(),
        current_region_key: request.current_region_key.clone(),
        current_region_name: request.current_region_name.clone(),
        current_row: request.current_row.clone(),
        current_seat_no: request.current_seat_no.clone(),
        desired_seats: request.desired_seats.clone(),
        contact: show_contact.then(|| request.contact.clone()),
        status,
        created_at: request.created_at.to_rfc3339(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::*;
    use crate::seat_swap::{
        domain::{
            SeatSwapCandidateStatus, SeatSwapContact, SeatSwapCurrentMatch, SeatSwapDesiredSeat,
            SeatSwapRequest, SeatSwapRequestStatus, SeatSwapUser,
        },
        ports::{
            current_match_port::CurrentSeatSwapMatchPort,
            seat_swap_repository::{SeatSwapConfirmation, SeatSwapRepository},
        },
    };

    const USER_A: Uuid = Uuid::from_u128(0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa);
    const USER_B: Uuid = Uuid::from_u128(0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb);

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

    struct StubSeatSwapRepository {
        requests: Vec<SeatSwapRequest>,
    }

    #[async_trait]
    impl SeatSwapRepository for StubSeatSwapRepository {
        async fn list_active_requests(
            &self,
            _match_id: i64,
        ) -> anyhow::Result<Vec<SeatSwapRequest>> {
            Ok(self.requests.clone())
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

        async fn find_confirmation(
            &self,
            _match_id: i64,
            _request_id: Uuid,
        ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
            Ok(None)
        }
    }

    fn request_fixture(
        request_id: u128,
        user_id: Uuid,
        current_region_key: &str,
        desired_region_keys: Vec<&str>,
    ) -> SeatSwapRequest {
        SeatSwapRequest {
            id: Uuid::from_u128(request_id),
            match_id: 574,
            user: SeatSwapUser {
                user_id,
                display_name: format!("用户{}", &user_id.to_string()[..4]),
            },
            current_region_key: current_region_key.to_string(),
            current_region_name: format!("{current_region_key}区"),
            current_row: "8".to_string(),
            current_seat_no: "15".to_string(),
            desired_seats: desired_region_keys
                .into_iter()
                .map(|region_key| SeatSwapDesiredSeat {
                    region_key: region_key.to_string(),
                    region_name: format!("{region_key}区"),
                    desired_row: None,
                    desired_seat_no: None,
                })
                .collect(),
            contact: SeatSwapContact::new(Some(format!("wx-{current_region_key}")), None)
                .expect("contact"),
            status: SeatSwapRequestStatus::Active,
            matched_request_id: None,
            created_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
        }
    }

    fn build_use_case_with_pool(requests: Vec<SeatSwapRequest>) -> GetCurrentSeatSwapUseCase {
        GetCurrentSeatSwapUseCase::new(
            Arc::new(StubSeatSwapRepository { requests }),
            Arc::new(StubCurrentMatchPort),
        )
    }

    #[tokio::test]
    async fn hides_contacts_for_display_only_candidates() {
        let use_case = build_use_case_with_pool(vec![
            request_fixture(1, USER_A, "A", vec!["B"]),
            request_fixture(2, USER_B, "C", vec!["A"]),
        ]);

        let view = use_case.execute(Some(USER_A)).await.expect("view");

        assert_eq!(
            view.candidates[0].status,
            SeatSwapCandidateStatus::DisplayOnly
        );
        assert!(view.candidates[0].contact.is_none());
    }

    #[tokio::test]
    async fn exposes_contacts_for_bidirectional_candidates() {
        let use_case = build_use_case_with_pool(vec![
            request_fixture(1, USER_A, "A", vec!["B"]),
            request_fixture(2, USER_B, "B", vec!["A"]),
        ]);

        let view = use_case.execute(Some(USER_A)).await.expect("view");

        assert_eq!(
            view.candidates[0].status,
            SeatSwapCandidateStatus::Communicable
        );
        assert!(view.candidates[0].contact.is_some());
    }
}
