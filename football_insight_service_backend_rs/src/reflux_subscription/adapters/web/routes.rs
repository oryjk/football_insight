use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::reflux_subscription::adapters::web::handlers::{
    RefluxSubscriptionWebState, create_reflux_subscription_order_handler,
    get_reflux_notification_email_handler, get_reflux_subscription_plans_handler,
    get_reflux_subscription_status_handler, update_reflux_notification_email_handler,
};

pub fn reflux_subscription_routes(state: Arc<RefluxSubscriptionWebState>) -> Router {
    Router::new()
        .route(
            "/api/v1/ticket-watch/reflux-subscriptions/plans",
            get(get_reflux_subscription_plans_handler),
        )
        .route(
            "/api/v1/ticket-watch/reflux-subscriptions/status",
            get(get_reflux_subscription_status_handler),
        )
        .route(
            "/api/v1/ticket-watch/reflux-subscriptions/order",
            post(create_reflux_subscription_order_handler),
        )
        .route(
            "/api/v1/ticket-watch/reflux-subscriptions/email",
            get(get_reflux_notification_email_handler).put(update_reflux_notification_email_handler),
        )
        .with_state(state)
}
