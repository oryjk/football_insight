use std::sync::Arc;

use super::handlers::{
    get_daily_topic_by_date, get_daily_topics, get_latest_daily_topic, get_topic_detail,
    get_topic_history, get_topic_timeline, record_topic_visit,
};
use crate::daily_topics::application::get_daily_topics::GetDailyTopics;
use axum::{
    Router,
    routing::{get, post},
};

pub fn daily_topics_routes(use_case: Arc<GetDailyTopics>) -> Router {
    Router::new()
        .route("/api/v1/daily-topics", get(get_daily_topics))
        .route("/api/v1/daily-topics/latest", get(get_latest_daily_topic))
        .route(
            "/api/v1/daily-topics/by-date/{date}",
            get(get_daily_topic_by_date),
        )
        .route("/api/v1/daily-topics/history", get(get_topic_history))
        .route("/api/v1/daily-topics/{id}", get(get_topic_detail))
        .route(
            "/api/v1/daily-topics/{id}/timeline",
            get(get_topic_timeline),
        )
        .route("/api/v1/daily-topics/{id}/visits", post(record_topic_visit))
        .with_state(use_case)
}

/// Same routes, plus login recognition so topic visits can record logged-in readers.
pub fn daily_topics_routes_with_auth(
    use_case: Arc<GetDailyTopics>,
    token_port: Arc<dyn crate::auth::ports::token_port::TokenPort>,
) -> Router {
    daily_topics_routes(use_case).layer(axum::Extension(token_port))
}
