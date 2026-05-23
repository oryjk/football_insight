use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::token_port::TokenPort,
    push_notification::{
        adapters::{
            persistence::postgres_device_token_repository::PostgresDeviceTokenRepository,
            web::{handlers::PushNotificationWebState, routes::push_notification_routes},
        },
        application::register_device_token::RegisterDeviceTokenUseCase,
    },
};

pub fn build_push_notification_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router {
    let repository = Arc::new(PostgresDeviceTokenRepository::new(pool));
    let state = Arc::new(PushNotificationWebState {
        register_use_case: Arc::new(RegisterDeviceTokenUseCase::new(repository)),
        token_port,
    });

    push_notification_routes(state)
}
