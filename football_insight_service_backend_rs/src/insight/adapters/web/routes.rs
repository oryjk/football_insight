use std::sync::Arc;

use axum::{Router, routing::get};

use crate::insight::application::{
    get_live_matches::GetLiveMatchesUseCase, get_live_overview::GetLiveOverviewUseCase,
    get_live_rankings::GetLiveRankingsUseCase, get_live_team_insights::GetLiveTeamInsightsUseCase,
    get_overview::GetOverviewUseCase, get_round_matches::GetRoundMatchesUseCase,
    get_round_overview::GetRoundOverviewUseCase, get_round_rankings::GetRoundRankingsUseCase,
    list_available_rounds::ListAvailableRoundsUseCase,
};

use super::handlers::{
    get_live_matches_handler, get_live_overview_handler, get_live_rankings_handler,
    get_live_team_insights_handler, get_overview_handler, get_round_matches_handler,
    get_round_overview_handler, get_round_rankings_handler, list_available_rounds_handler,
};

pub fn insight_routes(
    overview_use_case: Arc<GetOverviewUseCase>,
    live_use_case: Arc<GetLiveOverviewUseCase>,
    live_rankings_use_case: Arc<GetLiveRankingsUseCase>,
    live_team_insights_use_case: Arc<GetLiveTeamInsightsUseCase>,
    live_matches_use_case: Arc<GetLiveMatchesUseCase>,
    round_use_case: Arc<GetRoundOverviewUseCase>,
    round_rankings_use_case: Arc<GetRoundRankingsUseCase>,
    round_matches_use_case: Arc<GetRoundMatchesUseCase>,
    rounds_use_case: Arc<ListAvailableRoundsUseCase>,
) -> Router {
    Router::new()
        .route(
            "/api/v1/insights/overview",
            get(move || get_overview_handler(overview_use_case.clone())),
        )
        .route(
            "/api/v1/live/overview",
            get(move || get_live_overview_handler(live_use_case.clone())),
        )
        .route(
            "/api/v1/live/rankings",
            get(move || get_live_rankings_handler(live_rankings_use_case.clone())),
        )
        .route(
            "/api/v1/live/team-insights",
            get(move || get_live_team_insights_handler(live_team_insights_use_case.clone())),
        )
        .route(
            "/api/v1/live/matches",
            get(move || get_live_matches_handler(live_matches_use_case.clone())),
        )
        .route(
            "/api/v1/rounds",
            get(move |query| list_available_rounds_handler(query, rounds_use_case.clone())),
        )
        .route(
            "/api/v1/rounds/{season}/{round_number}/overview",
            get(move |path| get_round_overview_handler(path, round_use_case.clone())),
        )
        .route(
            "/api/v1/rounds/{season}/{round_number}/rankings",
            get(move |path| get_round_rankings_handler(path, round_rankings_use_case.clone())),
        )
        .route(
            "/api/v1/rounds/{season}/{round_number}/matches",
            get(move |path| get_round_matches_handler(path, round_matches_use_case.clone())),
        )
}
