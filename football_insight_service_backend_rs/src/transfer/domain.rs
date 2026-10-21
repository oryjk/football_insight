use chrono::{DateTime, Utc};

/// 提名意向：希望离队 / 希望留下 / 希望买入（本队旧将显示为"希望回归"）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransferIntent {
    Out,
    Stay,
    In,
}

impl TransferIntent {
    pub const ALL: [TransferIntent; 3] = [Self::Out, Self::Stay, Self::In];

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "out" => Some(Self::Out),
            "stay" => Some(Self::Stay),
            "in" => Some(Self::In),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Out => "out",
            Self::Stay => "stay",
            Self::In => "in",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoteValue {
    Up,
    Down,
}

impl VoteValue {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Up),
            -1 => Some(Self::Down),
            _ => None,
        }
    }

    pub fn as_i16(self) -> i16 {
        match self {
            Self::Up => 1,
            Self::Down => -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferWindow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub opens_at: DateTime<Utc>,
    pub closes_at: DateTime<Utc>,
}

impl TransferWindow {
    /// 开放区间左闭右开：`opens_at <= now < closes_at`。
    pub fn is_open(&self, now: DateTime<Utc>) -> bool {
        self.opens_at <= now && now < self.closes_at
    }
}

/// 球员相对某支球队的身份。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Membership {
    /// 本队现役
    Current,
    /// 本队旧将（名单里有本队 active = false 的记录），不论现在是否在别队现役
    Former,
    /// 从未在本队、现于其他队现役
    Elsewhere,
    /// 以上都不是（不在任何现役名单里，也没在本队效力过）
    Unknown,
}

/// `rows` 为该球员在名单表里的 (team_id, active) 全部记录。
pub fn membership_from(team_id: i64, rows: &[(i64, bool)]) -> Membership {
    if rows.iter().any(|&(team, active)| team == team_id && active) {
        Membership::Current
    } else if rows
        .iter()
        .any(|&(team, active)| team == team_id && !active)
    {
        Membership::Former
    } else if rows.iter().any(|&(_, active)| active) {
        Membership::Elsewhere
    } else {
        Membership::Unknown
    }
}

pub fn candidate_allowed(intent: TransferIntent, membership: Membership) -> bool {
    match intent {
        TransferIntent::Out | TransferIntent::Stay => membership == Membership::Current,
        TransferIntent::In => matches!(membership, Membership::Former | Membership::Elsewhere),
    }
}

/// 主教练与球员共用提名卡的 `player_id`；主教练编号从这里开始（见 f_i_team_coaches 迁移）。
pub const COACH_ID_MIN: i64 = 9_000_000_001;

pub fn is_coach_id(id: i64) -> bool {
    id >= COACH_ID_MIN
}

/// 主教练只能被提名「走 / 留」，且必须是本队现任。
pub fn coach_allowed(intent: TransferIntent, membership: Membership) -> bool {
    matches!(intent, TransferIntent::Out | TransferIntent::Stay)
        && membership == Membership::Current
}

pub fn is_returning(intent: TransferIntent, former_of_team: bool) -> bool {
    intent == TransferIntent::In && former_of_team
}

/// "走 / 留"卡上的球员已不在本队现役。
pub fn is_departed(intent: TransferIntent, active_in_team: bool) -> bool {
    matches!(intent, TransferIntent::Out | TransferIntent::Stay) && !active_in_team
}

/// 每人每期每种意向最多新建的提名卡数；为已存在的卡加票不占名额。
pub const NOMINATION_LIMIT: i64 = 3;

pub fn nomination_limit_reached(created: i64) -> bool {
    created >= NOMINATION_LIMIT
}

pub fn valid_share_code(code: &str) -> bool {
    code.len() == 8 && code.bytes().all(|b| b.is_ascii_alphanumeric())
}

pub fn generate_share_code() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::rng();
    (0..8)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferTeam {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardPlayer {
    pub player_id: i64,
    pub name: String,
    pub avatar_url: Option<String>,
    pub position: Option<String>,
    pub shirt_number: Option<i32>,
    pub current_team: Option<TransferTeam>,
    pub departed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NominationCard {
    pub share_code: String,
    pub intent: TransferIntent,
    pub returning: bool,
    pub player: CardPlayer,
    pub up: i64,
    pub down: i64,
    /// -1 / 0 / 1；游客恒为 0
    pub my_vote: i16,
    /// 一票都没有的卡为 None（不进榜单）
    pub rank: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidatePlayer {
    pub player_id: i64,
    pub name: String,
    pub shirt_number: Option<i32>,
    pub position: Option<String>,
    pub avatar_url: Option<String>,
    pub current_team: Option<TransferTeam>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoteTally {
    pub up: i64,
    pub down: i64,
    pub my_vote: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferCodeImage {
    pub image_base64: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TransferError {
    #[error("参数无效")]
    Invalid,
    #[error("请登录后继续")]
    Unauthorized,
    #[error("只能为你的主队投票")]
    NotFavoriteTeam,
    #[error("这张提名不存在")]
    NotFound,
    #[error("本期投票已结束")]
    WindowClosed,
    #[error("这位球员不能这样提名")]
    InvalidCandidate,
    #[error("本期这一类最多提名 3 位")]
    NominationLimit,
    /// 内部：分享码撞车，由 application 重试，不应返回给用户
    #[error("分享码冲突")]
    ShareCodeTaken,
    #[error("服务暂时不可用，请稍后重试")]
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 1, hour, 0, 0).unwrap()
    }

    #[test]
    fn intent_and_vote_round_trip_and_reject_unknown_values() {
        for intent in TransferIntent::ALL {
            assert_eq!(TransferIntent::parse(intent.as_str()), Some(intent));
        }
        assert_eq!(TransferIntent::parse("OUT"), None);
        assert_eq!(VoteValue::from_i16(1), Some(VoteValue::Up));
        assert_eq!(VoteValue::from_i16(-1), Some(VoteValue::Down));
        assert_eq!(VoteValue::from_i16(0), None);
        assert_eq!(VoteValue::Down.as_i16(), -1);
    }

    #[test]
    fn window_is_open_on_a_half_open_interval() {
        let window = TransferWindow {
            id: 1,
            code: "w".into(),
            name: "w".into(),
            opens_at: at(8),
            closes_at: at(10),
        };
        assert!(!window.is_open(at(7)));
        assert!(window.is_open(at(8)));
        assert!(window.is_open(at(9)));
        assert!(!window.is_open(at(10)));
    }

    #[test]
    fn membership_prefers_current_then_former_then_elsewhere() {
        assert_eq!(membership_from(1, &[(1, true)]), Membership::Current);
        assert_eq!(
            membership_from(1, &[(1, false), (2, true)]),
            Membership::Former
        );
        assert_eq!(membership_from(1, &[(2, true)]), Membership::Elsewhere);
        assert_eq!(membership_from(1, &[(2, false)]), Membership::Unknown);
        assert_eq!(membership_from(1, &[]), Membership::Unknown);
    }

    #[test]
    fn candidates_follow_the_three_intents() {
        use Membership::*;
        for (intent, allowed) in [
            (TransferIntent::Out, [true, false, false, false]),
            (TransferIntent::Stay, [true, false, false, false]),
            (TransferIntent::In, [false, true, true, false]),
        ] {
            let got = [Current, Former, Elsewhere, Unknown].map(|m| candidate_allowed(intent, m));
            assert_eq!(got, allowed, "{intent:?}");
        }
    }

    #[test]
    fn coaches_can_only_be_voted_out_or_kept_by_their_current_team() {
        use Membership::*;
        assert!(!is_coach_id(9_000_000_000));
        assert!(is_coach_id(COACH_ID_MIN));
        for (intent, allowed) in [
            (TransferIntent::Out, [true, false, false, false]),
            (TransferIntent::Stay, [true, false, false, false]),
            (TransferIntent::In, [false, false, false, false]),
        ] {
            let got = [Current, Former, Elsewhere, Unknown].map(|m| coach_allowed(intent, m));
            assert_eq!(got, allowed, "{intent:?}");
        }
    }

    #[test]
    fn returning_and_departed_flags() {
        assert!(is_returning(TransferIntent::In, true));
        assert!(!is_returning(TransferIntent::In, false));
        assert!(!is_returning(TransferIntent::Out, true));
        assert!(is_departed(TransferIntent::Out, false));
        assert!(is_departed(TransferIntent::Stay, false));
        assert!(!is_departed(TransferIntent::Stay, true));
        assert!(!is_departed(TransferIntent::In, false));
    }

    #[test]
    fn nomination_limit_is_three_new_cards() {
        assert!(!nomination_limit_reached(2));
        assert!(nomination_limit_reached(3));
        assert!(nomination_limit_reached(4));
    }

    #[test]
    fn share_codes_are_eight_base62_characters() {
        for _ in 0..50 {
            let code = generate_share_code();
            assert!(valid_share_code(&code), "{code}");
        }
        assert!(valid_share_code("aAzZ0199"));
        for bad in ["", "short", "toolong123", "with-dsh", "中文中文中文中文"] {
            assert!(!valid_share_code(bad), "{bad}");
        }
    }

    #[test]
    fn errors_have_user_facing_chinese_messages() {
        assert_eq!(
            TransferError::NotFavoriteTeam.to_string(),
            "只能为你的主队投票"
        );
        assert_eq!(TransferError::WindowClosed.to_string(), "本期投票已结束");
        assert_eq!(
            TransferError::NominationLimit.to_string(),
            "本期这一类最多提名 3 位"
        );
    }
}
