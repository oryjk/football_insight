use serde::{Deserialize, Serialize};

use crate::support::domain::support::{
    SupportMatchDetail, SupportMatchTeam, SupportProfileView, SupportTeamSummary,
    SupportViewerState,
};

#[derive(Debug, Serialize)]
pub struct SupportTeamDto {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_storage_url: Option<String>,
    pub rank_no: Option<i32>,
}

impl From<SupportTeamSummary> for SupportTeamDto {
    fn from(value: SupportTeamSummary) -> Self {
        Self {
            team_id: value.team_id,
            team_name: value.team_name,
            avatar_storage_url: value.avatar_storage_url,
            rank_no: value.rank_no,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SupportMatchTeamDto {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_storage_url: Option<String>,
    pub score: String,
    pub support_count: i64,
    pub support_share_pct: i32,
    pub season_support_rank: Option<i32>,
}

impl From<SupportMatchTeam> for SupportMatchTeamDto {
    fn from(value: SupportMatchTeam) -> Self {
        Self {
            team_id: value.team_id,
            team_name: value.team_name,
            avatar_storage_url: value.avatar_storage_url,
            score: value.score,
            support_count: value.support_count,
            support_share_pct: value.support_share_pct,
            season_support_rank: value.season_support_rank,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SupportViewerStateDto {
    pub favorite_team_id: Option<i64>,
    pub supported_team_id: Option<i64>,
    pub has_supported: bool,
    pub can_support: bool,
}

impl From<SupportViewerState> for SupportViewerStateDto {
    fn from(value: SupportViewerState) -> Self {
        Self {
            favorite_team_id: value.favorite_team_id,
            supported_team_id: value.supported_team_id,
            has_supported: value.has_supported,
            can_support: value.can_support,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SupportMatchDetailDto {
    pub match_id: i64,
    pub season: i32,
    pub round_number: i32,
    pub match_date: String,
    pub match_time: String,
    pub kickoff_at: String,
    pub status: String,
    pub support_window_status: String,
    pub countdown_seconds: i64,
    pub total_support_count: i64,
    pub home_team: SupportMatchTeamDto,
    pub away_team: SupportMatchTeamDto,
    pub viewer: SupportViewerStateDto,
}

impl From<SupportMatchDetail> for SupportMatchDetailDto {
    fn from(value: SupportMatchDetail) -> Self {
        Self {
            match_id: value.match_id,
            season: value.season,
            round_number: value.round_number,
            match_date: value.match_date,
            match_time: value.match_time,
            kickoff_at: value.kickoff_at.to_rfc3339(),
            status: value.status,
            support_window_status: value.support_window_status.as_str().to_string(),
            countdown_seconds: value.countdown_seconds,
            total_support_count: value.total_support_count,
            home_team: value.home_team.into(),
            away_team: value.away_team.into(),
            viewer: value.viewer.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SupportProfileDto {
    pub favorite_team: Option<SupportTeamDto>,
    pub next_match: Option<SupportMatchDetailDto>,
}

impl From<SupportProfileView> for SupportProfileDto {
    fn from(value: SupportProfileView) -> Self {
        Self {
            favorite_team: value.favorite_team.map(Into::into),
            next_match: value.next_match.map(Into::into),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetFavoriteTeamRequest {
    pub team_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct CastSupportVoteRequest {
    pub supported_team_id: i64,
}
