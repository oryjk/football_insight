use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeatSwapRequestStatus {
    Active,
    Matched,
    Cancelled,
}

impl SeatSwapRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Matched => "matched",
            Self::Cancelled => "cancelled",
        }
    }
}

impl TryFrom<&str> for SeatSwapRequestStatus {
    type Error = SeatSwapValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "active" => Ok(Self::Active),
            "matched" => Ok(Self::Matched),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(SeatSwapValidationError::InvalidStatus),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeatSwapCandidateStatus {
    Communicable,
    WaitingPeerConfirmation,
    PeerConfirmedMe,
    Matched,
    DisplayOnly,
}

impl SeatSwapCandidateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Communicable => "communicable",
            Self::WaitingPeerConfirmation => "waiting_peer_confirmation",
            Self::PeerConfirmedMe => "peer_confirmed_me",
            Self::Matched => "matched",
            Self::DisplayOnly => "display_only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeatSwapValidationError {
    ContactRequired,
    InvalidPhoneNumber,
    InvalidStatus,
}

#[derive(Debug, thiserror::Error)]
pub enum SeatSwapError {
    #[error("当前暂无可换座比赛")]
    NoCurrentMatch,
    #[error("已正式匹配的换座不能更新，请先按撤销流程处理")]
    MatchedRequestCannotUpdate,
    #[error("已正式匹配的换座需要提交撤销说明和截图")]
    MatchedRequestNeedsEvidenceCancellation,
    #[error("对方已经匹配成功，不能再确认")]
    TargetAlreadyMatched,
    #[error("换座对象已撤销，不能再确认")]
    TargetAlreadyCancelled,
    #[error("被别人抢先了，请刷新后重新匹配")]
    MatchAlreadyTaken,
    #[error("请选择有效的当前分区")]
    InvalidCurrentRegion,
    #[error("请输入当前排号")]
    CurrentRowRequired,
    #[error("请输入当前座号")]
    CurrentSeatNoRequired,
    #[error("请至少选择一个想换到的分区")]
    DesiredSeatRequired,
    #[error("请选择有效的目标分区")]
    InvalidDesiredRegion,
    #[error("请至少填写微信号或手机号")]
    ContactRequired,
    #[error("请输入 11 位手机号")]
    InvalidPhoneNumber,
    #[error("请先发布我的换座请求")]
    MyRequestRequired,
    #[error("换座对象不存在")]
    TargetRequestNotFound,
    #[error("只能确认当前比赛的换座对象")]
    ConfirmTargetNotCurrentMatch,
    #[error("当前没有待取消的匹配确认")]
    ConfirmationNotFound,
    #[error("请填写撤销说明")]
    CancelReasonRequired,
    #[error("请上传双方达成一致的截图")]
    CancelEvidenceRequired,
    #[error("截图数据无效")]
    InvalidCancelEvidence,
    #[error("换座请求不存在")]
    RequestNotFound,
    #[error("只能撤销已正式匹配的换座")]
    CanOnlyCancelMatchedRequest,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl SeatSwapError {
    pub fn is_bad_request(&self) -> bool {
        !matches!(self, Self::Internal(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapDesiredSeat {
    pub region_key: String,
    pub region_name: String,
    pub desired_row: Option<String>,
    pub desired_seat_no: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapContact {
    pub wechat_id: Option<String>,
    pub phone_number: Option<String>,
}

impl SeatSwapContact {
    pub fn new(
        wechat_id: Option<String>,
        phone_number: Option<String>,
    ) -> Result<Self, SeatSwapValidationError> {
        let wechat_id = normalize_optional(wechat_id);
        let phone_number = normalize_optional(phone_number);

        if let Some(phone_number) = phone_number.as_deref()
            && !is_mainland_phone_number(phone_number)
        {
            return Err(SeatSwapValidationError::InvalidPhoneNumber);
        }

        if wechat_id.is_none() && phone_number.is_none() {
            return Err(SeatSwapValidationError::ContactRequired);
        }

        Ok(Self {
            wechat_id,
            phone_number,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapUser {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapRequest {
    pub id: Uuid,
    pub match_id: i64,
    pub user: SeatSwapUser,
    pub current_region_key: String,
    pub current_region_name: String,
    pub current_row: String,
    pub current_seat_no: String,
    pub desired_seats: Vec<SeatSwapDesiredSeat>,
    pub contact: SeatSwapContact,
    pub seat_swap_notice_enabled: bool,
    pub status: SeatSwapRequestStatus,
    pub matched_request_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SeatSwapRequest {
    pub fn has_bidirectional_match_with(&self, peer: &SeatSwapRequest) -> bool {
        self.status != SeatSwapRequestStatus::Cancelled
            && peer.status != SeatSwapRequestStatus::Cancelled
            && self
                .desired_seats
                .iter()
                .any(|seat| seat.region_key == peer.current_region_key)
            && peer
                .desired_seats
                .iter()
                .any(|seat| seat.region_key == self.current_region_key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapCurrentMatch {
    pub match_id: i64,
    pub home_team_name: String,
    pub away_team_name: String,
    pub kickoff_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatSwapRegion {
    pub region_key: String,
    pub region_name: String,
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
}

fn is_mainland_phone_number(value: &str) -> bool {
    value.len() == 11 && value.starts_with('1') && value.chars().all(|item| item.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn seat_request(region_key: &str, desired_region_keys: Vec<&str>) -> SeatSwapRequest {
        SeatSwapRequest {
            id: Uuid::new_v4(),
            match_id: 574,
            user: SeatSwapUser {
                user_id: Uuid::new_v4(),
                display_name: "测试用户".to_string(),
                avatar_url: Some("https://img.example.com/avatar.png".to_string()),
            },
            current_region_key: region_key.to_string(),
            current_region_name: format!("{region_key}区"),
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
            contact: SeatSwapContact::new(Some("wechat".to_string()), None).expect("contact"),
            seat_swap_notice_enabled: false,
            status: SeatSwapRequestStatus::Active,
            matched_request_id: None,
            created_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 5, 18, 12, 0, 0).unwrap(),
        }
    }

    #[test]
    fn detects_bidirectional_region_match() {
        let mine = seat_request("A", vec!["B"]);
        let peer = seat_request("B", vec!["A"]);

        assert!(mine.has_bidirectional_match_with(&peer));
    }

    #[test]
    fn rejects_one_way_region_match() {
        let mine = seat_request("A", vec!["B"]);
        let peer = seat_request("B", vec!["C"]);

        assert!(!mine.has_bidirectional_match_with(&peer));
    }

    #[test]
    fn validates_at_least_one_contact_method() {
        let err = SeatSwapContact::new(None, None).expect_err("contact should be required");
        assert_eq!(err, SeatSwapValidationError::ContactRequired);
    }
}
