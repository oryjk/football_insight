use std::sync::Arc;

use axum::{Router, routing::post};

use crate::activity::adapters::web::handlers::{ActivityWebState, record_page_activity_handler};

pub fn activity_routes(state: Arc<ActivityWebState>) -> Router {
    Router::new()
        .route(
            "/api/v1/activity/page-view",
            post(record_page_activity_handler),
        )
        .with_state(state)
}
