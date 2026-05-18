use std::sync::Arc;

use axum::{Router, routing::{post, delete}};

use crate::push_notification::adapters::web::handlers::*;

pub fn push_notification_routes(state: Arc<PushNotificationWebState>) -> Router {
    Router::new()
        .route("/api/v1/push/register-token", post(register_token_handler))
        .route("/api/v1/push/unregister-token", delete(unregister_token_handler))
        .with_state(state)
}
