use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    activity::{
        adapters::{
            persistence::postgres_user_activity_repository::PostgresUserActivityRepository,
            web::{handlers::ActivityWebState, routes::activity_routes},
        },
        application::record_page_activity::RecordPageActivityUseCase,
    },
    auth::ports::token_port::TokenPort,
};

pub fn build_activity_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router {
    let repository = Arc::new(PostgresUserActivityRepository::new(pool));
    let state = Arc::new(ActivityWebState {
        record_page_activity_use_case: Arc::new(RecordPageActivityUseCase::new(repository)),
        token_port,
    });

    activity_routes(state)
}
