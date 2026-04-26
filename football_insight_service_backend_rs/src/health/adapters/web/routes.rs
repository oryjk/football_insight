use std::sync::Arc;

use axum::{Router, routing::get};

use crate::health::application::get_health::GetHealthUseCase;

use super::handlers::get_health_handler;

pub fn health_routes(use_case: Arc<GetHealthUseCase>) -> Router {
    Router::new().route(
        "/api/health",
        get(move || get_health_handler(use_case.clone())),
    )
}
