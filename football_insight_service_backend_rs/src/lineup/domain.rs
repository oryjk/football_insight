use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SquadTeam {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SquadPlayer {
    pub player_id: i64,
    pub name: String,
    pub shirt_number: Option<i32>,
    pub position: Option<Position>,
    pub avatar_url: Option<String>,
    pub last_season: i32,
    #[serde(skip)]
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SquadResponse {
    pub team: SquadTeam,
    pub active: Vec<SquadPlayer>,
    pub former: Vec<SquadPlayer>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineupSlot {
    pub slot: u8,
    pub player_id: Option<i64>,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Lineup {
    pub formation: String,
    pub slots: Vec<LineupSlot>,
    pub share_code: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupSaveError {
    #[error("阵型无效")]
    InvalidFormation,
    #[error("站位数据无效")]
    InvalidSlots,
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
