use serde::Serialize;

use crate::insight::domain::{
    insight_summary::InsightSummary,
    match_list::{MatchCard, MatchListView},
    overview::{InsightOverview, OverviewMatch, OverviewPlayer, OverviewStanding},
    rankings::{
        PlayerRankingCategory, PlayerRankingEntry, RankingsView, StandingsTable,
        StandingsTableEntry, TeamRankingCategory, TeamRankingEntry,
    },
    round_reference::RoundReference,
    team_insight::{
        AssistContribution, OpponentContribution, PlayerContribution, TeamInsight, TeamInsightTeam,
        TeamInsightsView,
    },
};

#[derive(Debug, Serialize)]
pub struct InsightOverviewResponseDto {
    pub view_kind: String,
    pub round_number: Option<i32>,
    pub current_season: i32,
    pub latest_scrape_finished_at: Option<String>,
    pub total_matches: i64,
    pub total_teams: i64,
    pub total_players: i64,
    pub player_ranking_categories: i64,
    pub team_ranking_categories: i64,
    pub standings_top: Vec<OverviewStandingDto>,
    pub recent_matches: Vec<OverviewMatchDto>,
    pub top_scorers: Vec<OverviewPlayerDto>,
    pub insight_summary: Option<InsightSummaryDto>,
}

#[derive(Debug, Serialize)]
pub struct OverviewStandingDto {
    pub rank_no: i32,
    pub team_id: i64,
    pub team_name: String,
    pub points: i32,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OverviewMatchDto {
    pub match_id: i64,
    pub round_number: i32,
    pub match_date: String,
    pub match_time: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub home_score: String,
    pub away_score: String,
}

#[derive(Debug, Serialize)]
pub struct OverviewPlayerDto {
    pub rank_no: i32,
    pub player_id: i64,
    pub player_name: String,
    pub team_name: String,
    pub score_value: String,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoundReferenceDto {
    pub season: i32,
    pub round_number: i32,
    pub finalized_at: Option<String>,
    pub status: String,
    pub total_matches: i32,
    pub completed_matches: i32,
}

#[derive(Debug, Serialize)]
pub struct RankingsViewResponseDto {
    pub view_kind: String,
    pub round_number: Option<i32>,
    pub current_season: i32,
    pub standings_tables: Vec<StandingsTableDto>,
    pub team_categories: Vec<TeamRankingCategoryDto>,
    pub player_categories: Vec<PlayerRankingCategoryDto>,
}

#[derive(Debug, Serialize)]
pub struct StandingsTableDto {
    pub slug: String,
    pub label: String,
    pub note: String,
    pub entries: Vec<StandingsTableEntryDto>,
}

#[derive(Debug, Serialize)]
pub struct StandingsTableEntryDto {
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

#[derive(Debug, Serialize)]
pub struct TeamRankingCategoryDto {
    pub slug: String,
    pub label: String,
    pub item_id: i32,
    pub entries: Vec<TeamRankingEntryDto>,
}

#[derive(Debug, Serialize)]
pub struct TeamRankingEntryDto {
    pub rank_no: i32,
    pub team_id: i64,
    pub team_name: String,
    pub score_value: String,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlayerRankingCategoryDto {
    pub slug: String,
    pub label: String,
    pub item_id: i32,
    pub entries: Vec<PlayerRankingEntryDto>,
}

#[derive(Debug, Serialize)]
pub struct PlayerRankingEntryDto {
    pub rank_no: i32,
    pub player_id: i64,
    pub player_name: String,
    pub team_id: i64,
    pub team_name: String,
    pub score_value: String,
    pub penalty_value: Option<String>,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MatchListResponseDto {
    pub view_kind: String,
    pub round_number: Option<i32>,
    pub current_season: i32,
    pub matches: Vec<MatchCardDto>,
}

#[derive(Debug, Serialize)]
pub struct TeamInsightsResponseDto {
    pub view_kind: String,
    pub round_number: Option<i32>,
    pub current_season: i32,
    pub teams: Vec<TeamInsightTeamDto>,
    pub insights: Vec<TeamInsightDto>,
}

#[derive(Debug, Serialize)]
pub struct TeamInsightTeamDto {
    pub team_id: i64,
    pub team_name: String,
    pub rank_no: i32,
    pub avatar_storage_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TeamInsightDto {
    pub team_id: i64,
    pub team_name: String,
    pub rank_no: i32,
    pub avatar_storage_url: Option<String>,
    pub goals_for_total: i32,
    pub goals_against_total: i32,
    pub goals_for_by_opponent: Vec<OpponentContributionDto>,
    pub goals_for_by_player: Vec<PlayerContributionDto>,
    pub assists_for_by_player: Vec<AssistContributionDto>,
    pub goals_against_by_opponent: Vec<OpponentContributionDto>,
}

#[derive(Debug, Serialize)]
pub struct OpponentContributionDto {
    pub opponent_team_id: i64,
    pub opponent_team_name: String,
    pub opponent_avatar_storage_url: Option<String>,
    pub goals: i32,
    pub share: f64,
}

#[derive(Debug, Serialize)]
pub struct PlayerContributionDto {
    pub player_id: Option<i64>,
    pub player_name: String,
    pub avatar_storage_url: Option<String>,
    pub goals: i32,
    pub share: f64,
}

#[derive(Debug, Serialize)]
pub struct AssistContributionDto {
    pub player_id: Option<i64>,
    pub player_name: String,
    pub avatar_storage_url: Option<String>,
    pub assists: i32,
    pub share: f64,
}

#[derive(Debug, Serialize)]
pub struct MatchCardDto {
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
    pub technical_stats: Vec<MatchTechnicalStatDto>,
}

#[derive(Debug, Serialize)]
pub struct MatchTechnicalStatDto {
    pub slug: String,
    pub label: String,
    pub home_value: i32,
    pub away_value: i32,
    pub unit: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InsightSummaryDto {
    pub headline: String,
    pub summary: String,
    pub bullets: Vec<String>,
    pub focus_match_id: Option<i64>,
}

impl From<InsightOverview> for InsightOverviewResponseDto {
    fn from(value: InsightOverview) -> Self {
        Self {
            view_kind: value.view_kind,
            round_number: value.round_number,
            current_season: value.current_season,
            latest_scrape_finished_at: value
                .latest_scrape_finished_at
                .map(|item| item.to_rfc3339()),
            total_matches: value.total_matches,
            total_teams: value.total_teams,
            total_players: value.total_players,
            player_ranking_categories: value.player_ranking_categories,
            team_ranking_categories: value.team_ranking_categories,
            standings_top: value.standings_top.into_iter().map(Into::into).collect(),
            recent_matches: value.recent_matches.into_iter().map(Into::into).collect(),
            top_scorers: value.top_scorers.into_iter().map(Into::into).collect(),
            insight_summary: value.insight_summary.map(Into::into),
        }
    }
}

impl From<OverviewStanding> for OverviewStandingDto {
    fn from(value: OverviewStanding) -> Self {
        Self {
            rank_no: value.rank_no,
            team_id: value.team_id,
            team_name: value.team_name,
            points: value.points,
            avatar_storage_url: value.avatar_storage_url,
        }
    }
}

impl From<OverviewMatch> for OverviewMatchDto {
    fn from(value: OverviewMatch) -> Self {
        Self {
            match_id: value.match_id,
            round_number: value.round_number,
            match_date: value.match_date,
            match_time: value.match_time,
            home_team_name: value.home_team_name,
            away_team_name: value.away_team_name,
            home_score: value.home_score,
            away_score: value.away_score,
        }
    }
}

impl From<OverviewPlayer> for OverviewPlayerDto {
    fn from(value: OverviewPlayer) -> Self {
        Self {
            rank_no: value.rank_no,
            player_id: value.player_id,
            player_name: value.player_name,
            team_name: value.team_name,
            score_value: value.score_value,
            avatar_storage_url: value.avatar_storage_url,
        }
    }
}

impl From<RoundReference> for RoundReferenceDto {
    fn from(value: RoundReference) -> Self {
        Self {
            season: value.season,
            round_number: value.round_number,
            finalized_at: value.finalized_at,
            status: value.status,
            total_matches: value.total_matches,
            completed_matches: value.completed_matches,
        }
    }
}

impl From<RankingsView> for RankingsViewResponseDto {
    fn from(value: RankingsView) -> Self {
        Self {
            view_kind: value.view_kind,
            round_number: value.round_number,
            current_season: value.current_season,
            standings_tables: value.standings_tables.into_iter().map(Into::into).collect(),
            team_categories: value.team_categories.into_iter().map(Into::into).collect(),
            player_categories: value
                .player_categories
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<StandingsTable> for StandingsTableDto {
    fn from(value: StandingsTable) -> Self {
        Self {
            slug: value.slug,
            label: value.label,
            note: value.note,
            entries: value.entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<StandingsTableEntry> for StandingsTableEntryDto {
    fn from(value: StandingsTableEntry) -> Self {
        Self {
            rank_no: value.rank_no,
            team_id: value.team_id,
            team_name: value.team_name,
            played: value.played,
            wins: value.wins,
            draws: value.draws,
            losses: value.losses,
            goals_for: value.goals_for,
            goals_against: value.goals_against,
            goal_difference: value.goal_difference,
            points: value.points,
            points_without_penalty: value.points_without_penalty,
            points_adjustment: value.points_adjustment,
            avatar_storage_url: value.avatar_storage_url,
        }
    }
}

impl From<TeamRankingCategory> for TeamRankingCategoryDto {
    fn from(value: TeamRankingCategory) -> Self {
        Self {
            slug: value.slug,
            label: value.label,
            item_id: value.item_id,
            entries: value.entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TeamRankingEntry> for TeamRankingEntryDto {
    fn from(value: TeamRankingEntry) -> Self {
        Self {
            rank_no: value.rank_no,
            team_id: value.team_id,
            team_name: value.team_name,
            score_value: value.score_value,
            avatar_storage_url: value.avatar_storage_url,
        }
    }
}

impl From<PlayerRankingCategory> for PlayerRankingCategoryDto {
    fn from(value: PlayerRankingCategory) -> Self {
        Self {
            slug: value.slug,
            label: value.label,
            item_id: value.item_id,
            entries: value.entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PlayerRankingEntry> for PlayerRankingEntryDto {
    fn from(value: PlayerRankingEntry) -> Self {
        Self {
            rank_no: value.rank_no,
            player_id: value.player_id,
            player_name: value.player_name,
            team_id: value.team_id,
            team_name: value.team_name,
            score_value: value.score_value,
            penalty_value: value.penalty_value,
            avatar_storage_url: value.avatar_storage_url,
        }
    }
}

impl From<MatchListView> for MatchListResponseDto {
    fn from(value: MatchListView) -> Self {
        Self {
            view_kind: value.view_kind,
            round_number: value.round_number,
            current_season: value.current_season,
            matches: value.matches.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TeamInsightsView> for TeamInsightsResponseDto {
    fn from(value: TeamInsightsView) -> Self {
        Self {
            view_kind: value.view_kind,
            round_number: value.round_number,
            current_season: value.current_season,
            teams: value.teams.into_iter().map(Into::into).collect(),
            insights: value.insights.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TeamInsightTeam> for TeamInsightTeamDto {
    fn from(value: TeamInsightTeam) -> Self {
        Self {
            team_id: value.team_id,
            team_name: value.team_name,
            rank_no: value.rank_no,
            avatar_storage_url: value.avatar_storage_url,
        }
    }
}

impl From<TeamInsight> for TeamInsightDto {
    fn from(value: TeamInsight) -> Self {
        Self {
            team_id: value.team_id,
            team_name: value.team_name,
            rank_no: value.rank_no,
            avatar_storage_url: value.avatar_storage_url,
            goals_for_total: value.goals_for_total,
            goals_against_total: value.goals_against_total,
            goals_for_by_opponent: value
                .goals_for_by_opponent
                .into_iter()
                .map(Into::into)
                .collect(),
            goals_for_by_player: value
                .goals_for_by_player
                .into_iter()
                .map(Into::into)
                .collect(),
            assists_for_by_player: value
                .assists_for_by_player
                .into_iter()
                .map(Into::into)
                .collect(),
            goals_against_by_opponent: value
                .goals_against_by_opponent
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<OpponentContribution> for OpponentContributionDto {
    fn from(value: OpponentContribution) -> Self {
        Self {
            opponent_team_id: value.opponent_team_id,
            opponent_team_name: value.opponent_team_name,
            opponent_avatar_storage_url: value.opponent_avatar_storage_url,
            goals: value.goals,
            share: value.share,
        }
    }
}

impl From<PlayerContribution> for PlayerContributionDto {
    fn from(value: PlayerContribution) -> Self {
        Self {
            player_id: value.player_id,
            player_name: value.player_name,
            avatar_storage_url: value.avatar_storage_url,
            goals: value.goals,
            share: value.share,
        }
    }
}

impl From<AssistContribution> for AssistContributionDto {
    fn from(value: AssistContribution) -> Self {
        Self {
            player_id: value.player_id,
            player_name: value.player_name,
            avatar_storage_url: value.avatar_storage_url,
            assists: value.assists,
            share: value.share,
        }
    }
}

impl From<MatchCard> for MatchCardDto {
    fn from(value: MatchCard) -> Self {
        Self {
            match_id: value.match_id,
            round_number: value.round_number,
            match_date: value.match_date,
            match_time: value.match_time,
            status: value.status,
            home_team_id: value.home_team_id,
            home_team_name: value.home_team_name,
            home_score: value.home_score,
            away_team_id: value.away_team_id,
            away_team_name: value.away_team_name,
            away_score: value.away_score,
            home_team_avatar: value.home_team_avatar,
            away_team_avatar: value.away_team_avatar,
            leisu_match_id: value.leisu_match_id,
            home_corners: value.home_corners,
            away_corners: value.away_corners,
            corner_source: value.corner_source,
            technical_stats: value.technical_stats.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<crate::insight::domain::match_list::MatchTechnicalStat> for MatchTechnicalStatDto {
    fn from(value: crate::insight::domain::match_list::MatchTechnicalStat) -> Self {
        Self {
            slug: value.slug,
            label: value.label,
            home_value: value.home_value,
            away_value: value.away_value,
            unit: value.unit,
        }
    }
}

impl From<InsightSummary> for InsightSummaryDto {
    fn from(value: InsightSummary) -> Self {
        Self {
            headline: value.headline,
            summary: value.summary,
            bullets: value.bullets,
            focus_match_id: value.focus_match_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MatchCardDto;
    use crate::insight::domain::match_list::{MatchCard, MatchTechnicalStat};

    #[test]
    fn match_card_dto_includes_corner_fields() {
        let dto = MatchCardDto::from(MatchCard {
            match_id: 288579,
            round_number: 1,
            match_date: "2026-03-08".to_string(),
            match_time: "19:00".to_string(),
            status: "finished".to_string(),
            home_team_id: 110645,
            home_team_name: "武汉三镇".to_string(),
            home_score: "0".to_string(),
            away_team_id: 136,
            away_team_name: "北京国安".to_string(),
            away_score: "2".to_string(),
            home_team_avatar: None,
            away_team_avatar: None,
            leisu_match_id: Some(4422785),
            home_corners: Some(4),
            away_corners: Some(7),
            corner_source: Some("leisu_detail".to_string()),
            technical_stats: vec![
                MatchTechnicalStat {
                    slug: "attacks".to_string(),
                    label: "进攻".to_string(),
                    home_value: 92,
                    away_value: 118,
                    unit: None,
                },
                MatchTechnicalStat {
                    slug: "corners".to_string(),
                    label: "角球".to_string(),
                    home_value: 4,
                    away_value: 7,
                    unit: None,
                },
            ],
        });

        assert_eq!(dto.leisu_match_id, Some(4422785));
        assert_eq!(dto.home_corners, Some(4));
        assert_eq!(dto.away_corners, Some(7));
        assert_eq!(dto.corner_source.as_deref(), Some("leisu_detail"));
        assert_eq!(dto.technical_stats.len(), 2);
        assert_eq!(dto.technical_stats[0].slug, "attacks");
        assert_eq!(dto.technical_stats[1].label, "角球");
    }
}
