use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
};
use serde::Deserialize;

use crate::insight::{
    adapters::web::dto::{
        InsightOverviewResponseDto, MatchListResponseDto, RankingsViewResponseDto,
        RoundReferenceDto, TeamInsightsResponseDto,
    },
    application::{
        get_live_matches::GetLiveMatchesUseCase, get_live_overview::GetLiveOverviewUseCase,
        get_live_rankings::GetLiveRankingsUseCase,
        get_live_team_insights::GetLiveTeamInsightsUseCase, get_overview::GetOverviewUseCase,
        get_round_matches::GetRoundMatchesUseCase, get_round_overview::GetRoundOverviewUseCase,
        get_round_rankings::GetRoundRankingsUseCase,
        list_available_rounds::ListAvailableRoundsUseCase,
    },
};

#[derive(Debug, Deserialize)]
pub struct RoundsQuery {
    pub season: i32,
}

pub async fn get_live_overview_handler(
    use_case: Arc<GetLiveOverviewUseCase>,
) -> Result<Json<InsightOverviewResponseDto>, (StatusCode, String)> {
    let overview = use_case
        .execute()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(overview.into()))
}

pub async fn get_round_overview_handler(
    Path((season, round_number)): Path<(i32, i32)>,
    use_case: Arc<GetRoundOverviewUseCase>,
) -> Result<Json<InsightOverviewResponseDto>, (StatusCode, String)> {
    let overview = use_case
        .execute(season, round_number)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(overview.into()))
}

pub async fn get_live_rankings_handler(
    use_case: Arc<GetLiveRankingsUseCase>,
) -> Result<Json<RankingsViewResponseDto>, (StatusCode, String)> {
    let rankings = use_case
        .execute()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(rankings.into()))
}

pub async fn get_live_team_insights_handler(
    use_case: Arc<GetLiveTeamInsightsUseCase>,
) -> Result<Json<TeamInsightsResponseDto>, (StatusCode, String)> {
    let insights = use_case
        .execute()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(insights.into()))
}

pub async fn get_round_rankings_handler(
    Path((season, round_number)): Path<(i32, i32)>,
    use_case: Arc<GetRoundRankingsUseCase>,
) -> Result<Json<RankingsViewResponseDto>, (StatusCode, String)> {
    let rankings = use_case
        .execute(season, round_number)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(rankings.into()))
}

pub async fn get_live_matches_handler(
    use_case: Arc<GetLiveMatchesUseCase>,
) -> Result<Json<MatchListResponseDto>, (StatusCode, String)> {
    let matches = use_case
        .execute()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let dto: MatchListResponseDto = matches.into();
    tracing::info!(match_count = dto.matches.len(), "实时比赛列表查询完成");
    Ok(Json(dto))
}

pub async fn get_round_matches_handler(
    Path((season, round_number)): Path<(i32, i32)>,
    use_case: Arc<GetRoundMatchesUseCase>,
) -> Result<Json<MatchListResponseDto>, (StatusCode, String)> {
    let matches = use_case
        .execute(season, round_number)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let dto: MatchListResponseDto = matches.into();
    tracing::info!(season, round_number, match_count = dto.matches.len(), "轮次比赛列表查询完成");
    Ok(Json(dto))
}

pub async fn list_available_rounds_handler(
    Query(query): Query<RoundsQuery>,
    use_case: Arc<ListAvailableRoundsUseCase>,
) -> Result<Json<Vec<RoundReferenceDto>>, (StatusCode, String)> {
    let rounds = use_case
        .execute(query.season)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(rounds.into_iter().map(Into::into).collect()))
}

pub async fn get_overview_handler(
    use_case: Arc<GetOverviewUseCase>,
) -> Result<Json<InsightOverviewResponseDto>, (StatusCode, String)> {
    let overview = use_case
        .execute()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let dto: InsightOverviewResponseDto = overview.into();
    tracing::info!(recent_match_count = dto.recent_matches.len(), standing_count = dto.standings_top.len(), "概览数据查询完成");
    Ok(Json(dto))
}
