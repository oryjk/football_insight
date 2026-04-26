use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct TeamInsightsView {
    pub view_kind: String,
    pub round_number: Option<i32>,
    pub current_season: i32,
    pub teams: Vec<TeamInsightTeam>,
    pub insights: Vec<TeamInsight>,
}

#[derive(Debug, Clone)]
pub struct TeamInsightTeam {
    pub team_id: i64,
    pub team_name: String,
    pub rank_no: i32,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TeamInsight {
    pub team_id: i64,
    pub team_name: String,
    pub rank_no: i32,
    pub avatar_storage_url: Option<String>,
    pub goals_for_total: i32,
    pub goals_against_total: i32,
    pub goals_for_by_opponent: Vec<OpponentContribution>,
    pub goals_for_by_player: Vec<PlayerContribution>,
    pub assists_for_by_player: Vec<AssistContribution>,
    pub goals_against_by_opponent: Vec<OpponentContribution>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpponentContribution {
    pub opponent_team_id: i64,
    pub opponent_team_name: String,
    pub opponent_avatar_storage_url: Option<String>,
    pub goals: i32,
    pub share: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerContribution {
    pub player_id: Option<i64>,
    pub player_name: String,
    pub avatar_storage_url: Option<String>,
    pub goals: i32,
    pub share: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssistContribution {
    pub player_id: Option<i64>,
    pub player_name: String,
    pub avatar_storage_url: Option<String>,
    pub assists: i32,
    pub share: f64,
}
