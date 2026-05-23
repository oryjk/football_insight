use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::token_port::TokenPort,
    support::{
        adapters::{
            persistence::postgres_support_repository::PostgresSupportRepository,
            web::{handlers::SupportWebState, routes::support_routes},
        },
        application::{
            cast_match_support_vote::CastMatchSupportVoteUseCase,
            get_match_support_detail::GetMatchSupportDetailUseCase,
            get_support_profile::GetSupportProfileUseCase,
            list_support_teams::ListSupportTeamsUseCase, set_favorite_team::SetFavoriteTeamUseCase,
        },
    },
};

pub fn build_support_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router {
    let repository = Arc::new(PostgresSupportRepository::new(pool));
    let state = Arc::new(SupportWebState {
        list_support_teams_use_case: Arc::new(ListSupportTeamsUseCase::new(repository.clone())),
        get_support_profile_use_case: Arc::new(GetSupportProfileUseCase::new(repository.clone())),
        set_favorite_team_use_case: Arc::new(SetFavoriteTeamUseCase::new(repository.clone())),
        get_match_support_detail_use_case: Arc::new(GetMatchSupportDetailUseCase::new(
            repository.clone(),
        )),
        cast_match_support_vote_use_case: Arc::new(CastMatchSupportVoteUseCase::new(repository)),
        token_port,
    });

    support_routes(state)
}
