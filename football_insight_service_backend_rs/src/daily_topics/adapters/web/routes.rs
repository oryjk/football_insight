use std::sync::Arc;

use super::handlers::{
    TelegraphInteractionsWebState, create_entry_comment_handler, delete_entry_comment_handler,
    dislike_entry_handler, get_comment_thread_handler, get_daily_topic_by_date, get_daily_topics,
    get_latest_daily_topic, get_topic_detail, get_topic_history, get_topic_timeline,
    like_entry_comment_handler, like_entry_handler, list_entry_comments_handler,
    record_topic_visit, undislike_entry_handler, unlike_entry_comment_handler,
    unlike_entry_handler,
};
use crate::daily_topics::application::{
    get_daily_topics::GetDailyTopics, telegraph_interactions::TelegraphInteractionsService,
};
use axum::{
    Router,
    routing::{delete, get, post, put},
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

pub fn telegraph_interactions_routes(
    service: Arc<TelegraphInteractionsService>,
    token_port: Arc<dyn crate::auth::ports::token_port::TokenPort>,
) -> Router {
    let state = TelegraphInteractionsWebState {
        service,
        token_port,
    };
    Router::new()
        .route(
            "/api/v1/daily-topics/{id}/entries/{entry_id}/like",
            put(like_entry_handler).delete(unlike_entry_handler),
        )
        .route(
            "/api/v1/daily-topics/{id}/entries/{entry_id}/dislike",
            put(dislike_entry_handler).delete(undislike_entry_handler),
        )
        .route(
            "/api/v1/daily-topics/{id}/entries/{entry_id}/comments",
            get(list_entry_comments_handler).post(create_entry_comment_handler),
        )
        .route(
            "/api/v1/daily-topics/{id}/entries/{entry_id}/comments/{comment_id}",
            delete(delete_entry_comment_handler),
        )
        .route(
            "/api/v1/daily-topics/{id}/entries/{entry_id}/comments/{comment_id}/thread",
            get(get_comment_thread_handler),
        )
        .route(
            "/api/v1/daily-topics/{id}/entries/{entry_id}/comments/{comment_id}/like",
            put(like_entry_comment_handler).delete(unlike_entry_comment_handler),
        )
        .with_state(state)
}

/// Same routes, plus login recognition so topic visits and entry reactions recognize logged-in readers.
pub fn daily_topics_routes_with_auth(
    use_case: Arc<GetDailyTopics>,
    token_port: Arc<dyn crate::auth::ports::token_port::TokenPort>,
) -> Router {
    daily_topics_routes(use_case).layer(axum::Extension(token_port))
}
