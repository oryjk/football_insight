use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post, put},
};

use super::handlers::{
    SeatSwapWebState, cancel_matched_seat_swap_handler, confirm_seat_swap_candidate_handler,
    delete_my_seat_swap_request_handler, get_current_seat_swap_handler,
    upsert_my_seat_swap_request_handler,
};

pub fn seat_swap_routes(state: Arc<SeatSwapWebState>) -> Router {
    Router::new()
        .route(
            "/api/v1/seat-swap/current",
            get(get_current_seat_swap_handler),
        )
        .route(
            "/api/v1/seat-swap/my-request",
            put(upsert_my_seat_swap_request_handler).delete(delete_my_seat_swap_request_handler),
        )
        .route(
            "/api/v1/seat-swap/matches/{target_request_id}/confirm",
            post(confirm_seat_swap_candidate_handler),
        )
        .route(
            "/api/v1/seat-swap/matches/{target_request_id}/cancel",
            post(cancel_matched_seat_swap_handler),
        )
        .with_state(state)
}
