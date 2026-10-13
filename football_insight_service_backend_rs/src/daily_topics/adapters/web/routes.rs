use std::sync::Arc;

use super::handlers::get_daily_topics;
use crate::daily_topics::application::get_daily_topics::GetDailyTopics;
use axum::{Router, routing::get};

pub fn daily_topics_routes(use_case: Arc<GetDailyTopics>) -> Router {
    Router::new()
        .route("/api/v1/daily-topics", get(get_daily_topics))
        .with_state(use_case)
}
