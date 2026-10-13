use std::sync::Arc;

use super::handlers::{get_daily_topics, get_topic_detail, get_topic_history};
use crate::daily_topics::application::get_daily_topics::GetDailyTopics;
use axum::{Router, routing::get};

pub fn daily_topics_routes(use_case: Arc<GetDailyTopics>) -> Router {
    Router::new()
        .route("/api/v1/daily-topics", get(get_daily_topics))
        .route("/api/v1/daily-topics/history", get(get_topic_history))
        .route("/api/v1/daily-topics/{id}", get(get_topic_detail))
        .with_state(use_case)
}
