use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use super::handlers::{
    TeamBoardWebState, add_team_board_comment_handler, create_team_board_post_handler,
    get_team_board_handler, toggle_team_board_post_like_handler,
};

pub fn team_board_routes(state: Arc<TeamBoardWebState>) -> Router {
    Router::new()
        .route(
            "/api/v1/team-boards/{team_id}",
            get(get_team_board_handler).post(create_team_board_post_handler),
        )
        .route(
            "/api/v1/team-boards/posts/{post_id}/comments",
            post(add_team_board_comment_handler),
        )
        .route(
            "/api/v1/team-boards/posts/{post_id}/likes",
            post(toggle_team_board_post_like_handler),
        )
        .with_state(state)
}
