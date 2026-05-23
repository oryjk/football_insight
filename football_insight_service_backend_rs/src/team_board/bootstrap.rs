use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::token_port::TokenPort,
    insight::ports::insight_query_repository::InsightQueryRepository,
    team_board::{
        adapters::{
            persistence::postgres_team_board_repository::PostgresTeamBoardRepository,
            web::{handlers::TeamBoardWebState, routes::team_board_routes},
        },
        application::{
            add_team_board_comment::AddTeamBoardCommentUseCase,
            create_team_board_post::CreateTeamBoardPostUseCase,
            get_team_board::GetTeamBoardUseCase,
            toggle_team_board_post_like::ToggleTeamBoardPostLikeUseCase,
        },
    },
};

pub fn build_team_board_routes(
    pool: PgPool,
    insight_repository: Arc<dyn InsightQueryRepository>,
    token_port: Arc<dyn TokenPort>,
) -> Router {
    let repository = Arc::new(PostgresTeamBoardRepository::new(pool));
    let state = Arc::new(TeamBoardWebState {
        get_team_board_use_case: Arc::new(GetTeamBoardUseCase::new(
            repository.clone(),
            insight_repository.clone(),
        )),
        create_team_board_post_use_case: Arc::new(CreateTeamBoardPostUseCase::new(
            repository.clone(),
            insight_repository,
        )),
        add_team_board_comment_use_case: Arc::new(AddTeamBoardCommentUseCase::new(
            repository.clone(),
        )),
        toggle_team_board_post_like_use_case: Arc::new(ToggleTeamBoardPostLikeUseCase::new(
            repository,
        )),
        token_port,
    });

    team_board_routes(state)
}
