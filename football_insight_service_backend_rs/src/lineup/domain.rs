#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    GK,
    DF,
    MF,
    FW,
}

impl Position {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "GK" => Some(Self::GK),
            "DF" => Some(Self::DF),
            "MF" => Some(Self::MF),
            "FW" => Some(Self::FW),
            _ => None,
        }
    }

    /// 排序权重：门将、后卫、中场、前锋，未知排最后。
    pub fn rank(position: Option<Self>) -> u8 {
        match position {
            Some(Self::GK) => 0,
            Some(Self::DF) => 1,
            Some(Self::MF) => 2,
            Some(Self::FW) => 3,
            None => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SquadTeam {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SquadPlayer {
    pub player_id: i64,
    pub name: String,
    pub shirt_number: Option<i32>,
    pub position: Option<Position>,
    pub avatar_url: Option<String>,
    pub last_season: i32,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupReadError {
    #[error("球队参数无效")]
    Invalid,
    #[error("球队不存在")]
    NotFound,
    #[error("名单暂不可用")]
    Unavailable,
}

use std::collections::HashSet;

pub const FORMATIONS: [&str; 7] = [
    "4-3-3", "4-4-2", "4-2-3-1", "3-5-2", "3-4-3", "5-3-2", "4-1-4-1",
];

#[derive(Debug, Clone, PartialEq)]
pub struct LineupSlot {
    pub slot: u8,
    pub player_id: Option<i64>,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lineup {
    pub formation: String,
    pub slots: Vec<LineupSlot>,
    pub share_code: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupSaveError {
    #[error("账号不可用，请重新登录")]
    Unauthorized,
    #[error("阵型无效")]
    InvalidFormation,
    #[error("站位数据无效")]
    InvalidSlots,
    #[error("发布阵容需要选满11名球员")]
    IncompleteLineup,
    #[error("球员不属于这支球队")]
    PlayerNotInSquad,
    #[error("同一球员不能重复上场")]
    DuplicatePlayer,
    #[error("球队不存在")]
    TeamNotFound,
    #[error("阵容暂不可用")]
    Unavailable,
}

/// 与球队名单无关的纯校验：阵型、11 个不重复的位置序号、坐标范围、球员不重复。
pub fn validate_lineup_input(formation: &str, slots: &[LineupSlot]) -> Result<(), LineupSaveError> {
    if !FORMATIONS.contains(&formation) {
        return Err(LineupSaveError::InvalidFormation);
    }
    if slots.len() != 11 {
        return Err(LineupSaveError::InvalidSlots);
    }
    let in_unit = |v: f64| v.is_finite() && (0.0..=1.0).contains(&v);
    let mut seen_slots = HashSet::new();
    for slot in slots {
        if slot.slot > 10 || !seen_slots.insert(slot.slot) || !in_unit(slot.x) || !in_unit(slot.y) {
            return Err(LineupSaveError::InvalidSlots);
        }
    }
    let mut seen_players = HashSet::new();
    for id in slots.iter().filter_map(|s| s.player_id) {
        if !seen_players.insert(id) {
            return Err(LineupSaveError::DuplicatePlayer);
        }
    }
    Ok(())
}

#[cfg(test)]
mod lineup_validation_tests {
    use super::*;

    fn slots(n: u8) -> Vec<LineupSlot> {
        (0..n)
            .map(|slot| LineupSlot {
                slot,
                player_id: None,
                x: 0.5,
                y: 0.5,
            })
            .collect()
    }

    #[test]
    fn accepts_eleven_unique_slots_for_every_formation() {
        for f in FORMATIONS {
            assert_eq!(validate_lineup_input(f, &slots(11)), Ok(()), "{f}");
        }
    }

    #[test]
    fn rejects_unknown_formation() {
        assert_eq!(
            validate_lineup_input("9-9-9", &slots(11)),
            Err(LineupSaveError::InvalidFormation)
        );
    }

    #[test]
    fn rejects_wrong_slot_count_duplicate_or_out_of_range_slot_index() {
        assert_eq!(
            validate_lineup_input("4-3-3", &slots(10)),
            Err(LineupSaveError::InvalidSlots)
        );
        assert_eq!(
            validate_lineup_input("4-3-3", &slots(12)),
            Err(LineupSaveError::InvalidSlots)
        );
        let mut dup = slots(11);
        dup[3].slot = 2;
        assert_eq!(
            validate_lineup_input("4-3-3", &dup),
            Err(LineupSaveError::InvalidSlots)
        );
        let mut high = slots(11);
        high[10].slot = 11;
        assert_eq!(
            validate_lineup_input("4-3-3", &high),
            Err(LineupSaveError::InvalidSlots)
        );
    }

    #[test]
    fn rejects_coordinates_outside_unit_square_or_not_finite() {
        for (x, y) in [
            (-0.01, 0.5),
            (1.01, 0.5),
            (0.5, -0.01),
            (0.5, 1.01),
            (f64::NAN, 0.5),
            (0.5, f64::INFINITY),
        ] {
            let mut s = slots(11);
            s[0].x = x;
            s[0].y = y;
            assert_eq!(
                validate_lineup_input("4-3-3", &s),
                Err(LineupSaveError::InvalidSlots),
                "{x},{y}"
            );
        }
    }

    #[test]
    fn rejects_the_same_player_twice() {
        let mut s = slots(11);
        s[0].player_id = Some(7);
        s[5].player_id = Some(7);
        assert_eq!(
            validate_lineup_input("4-3-3", &s),
            Err(LineupSaveError::DuplicatePlayer)
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupPublicError {
    #[error("参数无效")]
    Invalid,
    #[error("账号不可用，请重新登录")]
    Unauthorized,
    #[error("阵容或球队不存在")]
    NotFound,
    #[error("阵容暂不可用")]
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineupCodeImage {
    pub image_base64: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineupSort {
    Latest,
    Likes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicLineupQuery {
    pub sort: LineupSort,
    pub page: u32,
    pub page_size: u32,
}

impl PublicLineupQuery {
    pub fn parse(sort: &str, page: u32, page_size: u32) -> Result<Self, LineupPublicError> {
        let sort = match sort {
            "latest" => LineupSort::Latest,
            "likes" => LineupSort::Likes,
            _ => return Err(LineupPublicError::Invalid),
        };
        if page == 0 || !(1..=50).contains(&page_size) {
            return Err(LineupPublicError::Invalid);
        }
        Ok(Self {
            sort,
            page,
            page_size,
        })
    }

    pub fn offset(self) -> i64 {
        i64::from(self.page - 1) * i64::from(self.page_size)
    }
}

pub fn valid_share_code(code: &str) -> bool {
    code.len() == 8 && code.bytes().all(|b| b.is_ascii_alphanumeric())
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineupAuthor {
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineupLikeState {
    pub likes_count: i64,
    pub liked_by_me: bool,
    pub disliked_by_me: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineupReaction {
    Like,
    Dislike,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublicLineupItem {
    pub share_code: String,
    pub formation: String,
    pub author: LineupAuthor,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub likes: LineupLikeState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublicLineupPage {
    pub team: SquadTeam,
    pub items: Vec<PublicLineupItem>,
    pub total: i64,
    pub query: PublicLineupQuery,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SharedLineup {
    pub item: PublicLineupItem,
    pub team: SquadTeam,
    pub slots: Vec<LineupSlot>,
    pub players: Vec<SquadPlayer>,
}

#[cfg(test)]
mod public_validation_tests {
    use super::*;
    #[test]
    fn public_query_rejects_unknown_sort_and_invalid_paging() {
        for (sort, page, size) in [
            ("bad", 1, 20),
            ("latest", 0, 20),
            ("likes", 1, 0),
            ("likes", 1, 51),
        ] {
            assert_eq!(
                PublicLineupQuery::parse(sort, page, size),
                Err(LineupPublicError::Invalid)
            );
        }
        assert_eq!(
            PublicLineupQuery::parse("likes", 2, 50).unwrap().offset(),
            50
        );
    }
    #[test]
    fn shared_codes_are_exactly_eight_ascii_base62_characters() {
        assert!(valid_share_code("aAzZ0199"));
        for code in ["", "short", "123456789", "1234/678", "1234é78"] {
            assert!(!valid_share_code(code));
        }
    }
}

/// 私密编辑快照；没有分享码，也不参与任何公开查询。
#[derive(Debug, Clone, PartialEq)]
pub struct LineupDraft {
    pub formation: String,
    pub slots: Vec<LineupSlot>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineupEditor {
    pub lineup: Option<Lineup>,
    pub draft: Option<LineupDraft>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishedLineup {
    pub lineup: Lineup,
    pub draft: LineupDraft,
}

pub fn validate_publish_input(
    formation: &str,
    slots: &[LineupSlot],
) -> Result<(), LineupSaveError> {
    validate_lineup_input(formation, slots)?;
    if slots.iter().any(|slot| slot.player_id.is_none()) {
        return Err(LineupSaveError::IncompleteLineup);
    }
    Ok(())
}
