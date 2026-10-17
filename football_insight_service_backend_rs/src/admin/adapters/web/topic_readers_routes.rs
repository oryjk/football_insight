use super::{
    product_handlers::product_no_store,
    topic_readers_handlers::{self, AdminTopicReadersWebState},
};
use crate::admin::application::{
    admin_auth_service::AdminAuthService, admin_topic_readers_service::AdminTopicReadersService,
};
use axum::{Router, routing::get};
use std::sync::Arc;

pub fn admin_topic_readers_routes(
    service: Arc<AdminTopicReadersService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    Router::new()
        .route("/api/v1/admin/daily-topics/topics", get(topic_readers_handlers::topics))
        .route(
            "/api/v1/admin/daily-topics/{team_id}/{topic_id}/readers",
            get(topic_readers_handlers::readers),
        )
        .with_state(AdminTopicReadersWebState { service, auth_service })
        .layer(axum::middleware::map_response(product_no_store))
}
