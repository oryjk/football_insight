use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::match_id_unlock::adapters::web::handlers::{
    MatchIdUnlockWebState, create_match_id_order_handler, get_match_id_entitlement_handler,
};

pub fn match_id_unlock_routes(state: Arc<MatchIdUnlockWebState>) -> Router {
    Router::new()
        .route(
            "/api/v1/match-id/entitlement",
            get(get_match_id_entitlement_handler),
        )
        .route(
            "/api/v1/match-id/order",
            post(create_match_id_order_handler),
        )
        .with_state(state)
}
