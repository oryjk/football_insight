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
