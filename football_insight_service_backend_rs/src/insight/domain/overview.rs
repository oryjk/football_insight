use chrono::{DateTime, Utc};

use crate::insight::domain::insight_summary::InsightSummary;

#[derive(Debug, Clone)]
pub struct InsightOverview {
    pub view_kind: String,
    pub round_number: Option<i32>,
    pub current_season: i32,
    pub latest_scrape_finished_at: Option<DateTime<Utc>>,
    pub total_matches: i64,
    pub total_teams: i64,
    pub total_players: i64,
    pub player_ranking_categories: i64,
    pub team_ranking_categories: i64,
    pub standings_top: Vec<OverviewStanding>,
    pub recent_matches: Vec<OverviewMatch>,
    pub top_scorers: Vec<OverviewPlayer>,
    pub insight_summary: Option<InsightSummary>,
}

#[derive(Debug, Clone)]
pub struct OverviewStanding {
    pub rank_no: i32,
    pub team_id: i64,
    pub team_name: String,
    pub points: i32,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OverviewMatch {
    pub match_id: i64,
    pub round_number: i32,
    pub match_date: String,
    pub match_time: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub home_score: String,
    pub away_score: String,
}

#[derive(Debug, Clone)]
pub struct OverviewPlayer {
    pub rank_no: i32,
    pub player_id: i64,
    pub player_name: String,
    pub team_name: String,
    pub score_value: String,
    pub avatar_storage_url: Option<String>,
}
