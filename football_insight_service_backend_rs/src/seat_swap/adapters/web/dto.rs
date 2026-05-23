use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::seat_swap::{
    application::{
        cancel_matched_seat_swap::CancelMatchedSeatSwapInput,
        get_current_seat_swap::{SeatSwapCandidateView, SeatSwapCurrentView, SeatSwapRequestView},
        upsert_my_seat_swap_request::{UpsertDesiredSeatInput, UpsertMySeatSwapRequestInput},
    },
    domain::{SeatSwapContact, SeatSwapCurrentMatch, SeatSwapDesiredSeat, SeatSwapError},
    ports::evidence_storage_port::SeatSwapEvidenceUpload,
};

#[derive(Debug, Serialize)]
pub struct SeatSwapCurrentResponse {
    pub available: bool,
    pub current_match: Option<SeatSwapCurrentMatchDto>,
    pub my_request: Option<SeatSwapRequestDto>,
    pub candidates: Vec<SeatSwapCandidateDto>,
}

#[derive(Debug, Serialize)]
pub struct SeatSwapCurrentMatchDto {
    pub match_id: i64,
    pub home_team_name: String,
    pub away_team_name: String,
    pub kickoff_at: String,
}

#[derive(Debug, Serialize)]
pub struct SeatSwapRequestDto {
    pub request_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub current_region_key: String,
    pub current_region_name: String,
    pub current_row: String,
    pub current_seat_no: String,
    pub desired_seats: Vec<SeatSwapDesiredSeatDto>,
    pub contact: Option<SeatSwapContactDto>,
    pub status: String,
    pub created_at: String,
    pub seat_swap_notice_enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct SeatSwapCandidateDto {
    pub request_id: Uuid,
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub current_region_key: String,
    pub current_region_name: String,
    pub current_row: String,
    pub current_seat_no: String,
    pub desired_seats: Vec<SeatSwapDesiredSeatDto>,
    pub contact: Option<SeatSwapContactDto>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatSwapDesiredSeatDto {
    pub region_key: String,
    pub region_name: String,
    pub desired_row: Option<String>,
    pub desired_seat_no: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SeatSwapContactDto {
    pub wechat_id: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSeatSwapRequestDto {
    pub current_region_key: String,
    pub current_region_name: String,
    pub current_row: String,
    pub current_seat_no: String,
    pub wechat_id: Option<String>,
    pub phone_number: Option<String>,
    pub seat_swap_notice_enabled: Option<bool>,
    pub mini_program_notice_enabled: Option<bool>,
    pub desired_seats: Vec<SeatSwapDesiredSeatDto>,
}

#[derive(Debug, Deserialize)]
pub struct CancelMatchedSeatSwapRequestDto {
    pub reason: String,
    pub evidence_file_name: String,
    pub evidence_content_type: String,
    pub evidence_base64: String,
}

impl From<SeatSwapCurrentView> for SeatSwapCurrentResponse {
    fn from(value: SeatSwapCurrentView) -> Self {
        Self {
            available: value.available,
            current_match: value.current_match.map(Into::into),
            my_request: value.my_request.map(Into::into),
            candidates: value.candidates.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<SeatSwapCurrentMatch> for SeatSwapCurrentMatchDto {
    fn from(value: SeatSwapCurrentMatch) -> Self {
        Self {
            match_id: value.match_id,
            home_team_name: value.home_team_name,
            away_team_name: value.away_team_name,
            kickoff_at: value.kickoff_at,
        }
    }
}

impl From<SeatSwapRequestView> for SeatSwapRequestDto {
    fn from(value: SeatSwapRequestView) -> Self {
        Self {
            request_id: value.request_id,
            user_id: value.user_id,
            display_name: value.display_name,
            avatar_url: value.avatar_url,
            current_region_key: value.current_region_key,
            current_region_name: value.current_region_name,
            current_row: value.current_row,
            current_seat_no: value.current_seat_no,
            desired_seats: value.desired_seats.into_iter().map(Into::into).collect(),
            contact: value.contact.map(Into::into),
            status: value.status,
            created_at: value.created_at,
            seat_swap_notice_enabled: value.seat_swap_notice_enabled,
        }
    }
}

impl From<SeatSwapCandidateView> for SeatSwapCandidateDto {
    fn from(value: SeatSwapCandidateView) -> Self {
        Self {
            request_id: value.request_id,
            user_id: value.user_id,
            display_name: value.display_name,
            avatar_url: value.avatar_url,
            current_region_key: value.current_region_key,
            current_region_name: value.current_region_name,
            current_row: value.current_row,
            current_seat_no: value.current_seat_no,
            desired_seats: value.desired_seats.into_iter().map(Into::into).collect(),
            contact: value.contact.map(Into::into),
            status: value.status.as_str().to_string(),
            created_at: value.created_at,
        }
    }
}

impl From<SeatSwapDesiredSeat> for SeatSwapDesiredSeatDto {
    fn from(value: SeatSwapDesiredSeat) -> Self {
        Self {
            region_key: value.region_key,
            region_name: value.region_name,
            desired_row: value.desired_row,
            desired_seat_no: value.desired_seat_no,
        }
    }
}

impl From<SeatSwapContact> for SeatSwapContactDto {
    fn from(value: SeatSwapContact) -> Self {
        Self {
            wechat_id: value.wechat_id,
            phone_number: value.phone_number,
        }
    }
}

impl From<UpsertSeatSwapRequestDto> for UpsertMySeatSwapRequestInput {
    fn from(value: UpsertSeatSwapRequestDto) -> Self {
        Self {
            current_region_key: value.current_region_key,
            current_region_name: value.current_region_name,
            current_row: value.current_row,
            current_seat_no: value.current_seat_no,
            wechat_id: value.wechat_id,
            phone_number: value.phone_number,
            mini_program_notice_enabled: value
                .seat_swap_notice_enabled
                .or(value.mini_program_notice_enabled)
                .unwrap_or(false),
            desired_seats: value
                .desired_seats
                .into_iter()
                .map(|seat| UpsertDesiredSeatInput {
                    region_key: seat.region_key,
                    region_name: seat.region_name,
                    desired_row: seat.desired_row,
                    desired_seat_no: seat.desired_seat_no,
                })
                .collect(),
        }
    }
}

impl CancelMatchedSeatSwapRequestDto {
    pub fn into_input(
        self,
        target_request_id: Uuid,
    ) -> Result<CancelMatchedSeatSwapInput, SeatSwapError> {
        let bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            self.evidence_base64,
        )
        .map_err(|_| SeatSwapError::InvalidCancelEvidence)?;
        Ok(CancelMatchedSeatSwapInput {
            target_request_id,
            reason: self.reason,
            evidence: SeatSwapEvidenceUpload {
                file_name: self.evidence_file_name,
                content_type: self.evidence_content_type,
                bytes,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::UpsertSeatSwapRequestDto;
    use crate::seat_swap::application::upsert_my_seat_swap_request::UpsertMySeatSwapRequestInput;

    #[test]
    fn prefers_canonical_notice_field_when_deserializing() {
        let dto: UpsertSeatSwapRequestDto = serde_json::from_value(json!({
            "current_region_key": "A",
            "current_region_name": "A区",
            "current_row": "8",
            "current_seat_no": "15",
            "seat_swap_notice_enabled": true,
            "mini_program_notice_enabled": false,
            "desired_seats": [
                {
                    "region_key": "B",
                    "region_name": "B区"
                }
            ]
        }))
        .expect("deserialize dto");

        let input: UpsertMySeatSwapRequestInput = dto.into();
        assert!(input.mini_program_notice_enabled);
    }

    #[test]
    fn keeps_legacy_notice_field_compatible_when_deserializing() {
        let dto: UpsertSeatSwapRequestDto = serde_json::from_value(json!({
            "current_region_key": "A",
            "current_region_name": "A区",
            "current_row": "8",
            "current_seat_no": "15",
            "mini_program_notice_enabled": true,
            "desired_seats": [
                {
                    "region_key": "B",
                    "region_name": "B区"
                }
            ]
        }))
        .expect("deserialize dto");

        let input: UpsertMySeatSwapRequestInput = dto.into();
        assert!(input.mini_program_notice_enabled);
    }
}
