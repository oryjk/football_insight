use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchListView {
    pub view_kind: String,
    pub round_number: Option<i32>,
    pub current_season: i32,
    pub matches: Vec<MatchCard>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchTechnicalStat {
    pub slug: String,
    pub label: String,
    pub home_value: i32,
    pub away_value: i32,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchCard {
    pub match_id: i64,
    pub round_number: i32,
    pub match_date: String,
    pub match_time: String,
    pub status: String,
    pub home_team_id: i64,
    pub home_team_name: String,
    pub home_score: String,
    pub away_team_id: i64,
    pub away_team_name: String,
    pub away_score: String,
    pub home_team_avatar: Option<String>,
    pub away_team_avatar: Option<String>,
    pub leisu_match_id: Option<i64>,
    pub home_corners: Option<i32>,
    pub away_corners: Option<i32>,
    pub corner_source: Option<String>,
    pub technical_stats: Vec<MatchTechnicalStat>,
}
