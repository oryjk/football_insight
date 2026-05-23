use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::insight::{
    adapters::{
        persistence::postgres_insight_query_repository::PostgresInsightQueryRepository,
        web::routes::insight_routes,
    },
    application::{
        get_live_matches::GetLiveMatchesUseCase, get_live_overview::GetLiveOverviewUseCase,
        get_live_rankings::GetLiveRankingsUseCase,
        get_live_team_insights::GetLiveTeamInsightsUseCase, get_overview::GetOverviewUseCase,
        get_round_matches::GetRoundMatchesUseCase, get_round_overview::GetRoundOverviewUseCase,
        get_round_rankings::GetRoundRankingsUseCase,
        list_available_rounds::ListAvailableRoundsUseCase,
    },
    ports::insight_query_repository::InsightQueryRepository,
};

pub struct InsightBootstrap {
    pub routes: Router,
    pub repository: Arc<dyn InsightQueryRepository>,
}

pub fn build_insight(pool: PgPool) -> InsightBootstrap {
    let repository: Arc<dyn InsightQueryRepository> =
        Arc::new(PostgresInsightQueryRepository::new(pool));

    InsightBootstrap {
        routes: insight_routes(
            Arc::new(GetOverviewUseCase::new(repository.clone())),
            Arc::new(GetLiveOverviewUseCase::new(repository.clone())),
            Arc::new(GetLiveRankingsUseCase::new(repository.clone())),
            Arc::new(GetLiveTeamInsightsUseCase::new(repository.clone())),
            Arc::new(GetLiveMatchesUseCase::new(repository.clone())),
            Arc::new(GetRoundOverviewUseCase::new(repository.clone())),
            Arc::new(GetRoundRankingsUseCase::new(repository.clone())),
            Arc::new(GetRoundMatchesUseCase::new(repository.clone())),
            Arc::new(ListAvailableRoundsUseCase::new(repository.clone())),
        ),
        repository,
    }
}
