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
    /// Completed league matches, oldest first, at most six results.
    pub recent_form: Vec<MatchOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchOutcome {
    Win,
    Draw,
    Loss,
}

impl MatchOutcome {
    pub fn from_scores(goals_for: &str, goals_against: &str) -> Option<Self> {
        let goals_for = goals_for.trim().parse::<u32>().ok()?;
        let goals_against = goals_against.trim().parse::<u32>().ok()?;
        Some(match goals_for.cmp(&goals_against) {
            std::cmp::Ordering::Greater => Self::Win,
            std::cmp::Ordering::Equal => Self::Draw,
            std::cmp::Ordering::Less => Self::Loss,
        })
    }
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
    pub technical_stats: Vec<crate::insight::domain::match_list::MatchTechnicalStat>,
    pub home_corners: Option<i32>,
    pub away_corners: Option<i32>,
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

#[cfg(test)]
mod tests {
    use super::MatchOutcome;

    #[test]
    fn team_form_compares_numeric_scores_from_the_teams_perspective() {
        assert_eq!(
            MatchOutcome::from_scores("10", "2"),
            Some(MatchOutcome::Win)
        );
        assert_eq!(
            MatchOutcome::from_scores("0", "0"),
            Some(MatchOutcome::Draw)
        );
        assert_eq!(
            MatchOutcome::from_scores("1", "3"),
            Some(MatchOutcome::Loss)
        );
    }

    #[test]
    fn team_form_does_not_turn_missing_or_invalid_scores_into_draws() {
        for score in ["", "-", "-1", "1.5", "abc", "99999999999999999999"] {
            assert_eq!(MatchOutcome::from_scores(score, "0"), None);
            assert_eq!(MatchOutcome::from_scores("0", score), None);
        }
    }
}
