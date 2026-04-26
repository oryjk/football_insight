use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, put},
};

use crate::support::adapters::web::handlers::{
    SupportWebState, cast_support_vote_handler, get_match_support_detail_handler,
    get_support_profile_handler, list_support_teams_handler, set_favorite_team_handler,
};

pub fn support_routes(state: Arc<SupportWebState>) -> Router {
    Router::new()
        .route("/api/v1/support/teams", get(list_support_teams_handler))
        .route("/api/v1/support/profile", get(get_support_profile_handler))
        .route(
            "/api/v1/support/favorite-team",
            put(set_favorite_team_handler),
        )
        .route(
            "/api/v1/support/matches/{match_id}",
            get(get_match_support_detail_handler),
        )
        .route(
            "/api/v1/support/matches/{match_id}/votes",
            post(cast_support_vote_handler),
        )
        .with_state(state)
}
