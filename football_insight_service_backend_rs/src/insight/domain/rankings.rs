#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingsView {
    pub view_kind: String,
    pub round_number: Option<i32>,
    pub current_season: i32,
    pub standings_tables: Vec<StandingsTable>,
    pub team_categories: Vec<TeamRankingCategory>,
    pub player_categories: Vec<PlayerRankingCategory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandingsTable {
    pub slug: String,
    pub label: String,
    pub note: String,
    pub entries: Vec<StandingsTableEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandingsTableEntry {
    pub rank_no: i32,
    pub team_id: i64,
    pub team_name: String,
    pub played: i32,
    pub wins: i32,
    pub draws: i32,
    pub losses: i32,
    pub goals_for: i32,
    pub goals_against: i32,
    pub goal_difference: i32,
    pub points: i32,
    pub points_without_penalty: i32,
    pub points_adjustment: i32,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamRankingCategory {
    pub slug: String,
    pub label: String,
    pub item_id: i32,
    pub entries: Vec<TeamRankingEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamRankingEntry {
    pub rank_no: i32,
    pub team_id: i64,
    pub team_name: String,
    pub score_value: String,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRankingCategory {
    pub slug: String,
    pub label: String,
    pub item_id: i32,
    pub entries: Vec<PlayerRankingEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRankingEntry {
    pub rank_no: i32,
    pub player_id: i64,
    pub player_name: String,
    pub team_id: i64,
    pub team_name: String,
    pub score_value: String,
    pub penalty_value: Option<String>,
    pub avatar_storage_url: Option<String>,
}
