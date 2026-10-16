use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    activity::{
        adapters::{
            persistence::{
                postgres_paywall_event_repository::PostgresPaywallEventRepository,
                postgres_user_activity_repository::PostgresUserActivityRepository,
            },
            web::{handlers::ActivityWebState, routes::activity_routes},
        },
        application::{
            record_page_activity::RecordPageActivityUseCase,
            record_paywall_events::RecordPaywallEvents,
        },
    },
    auth::ports::token_port::TokenPort,
};

pub fn build_activity_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router {
    // Dedicated, bounded pool: paywall measurement cannot exhaust the business connection pool.
    let paywall_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_millis(100))
        .connect_lazy_with((*pool.connect_options()).clone());
    let repository = Arc::new(PostgresUserActivityRepository::new(pool));
    let state = Arc::new(ActivityWebState {
        record_page_activity_use_case: Arc::new(RecordPageActivityUseCase::new(repository)),
        record_paywall_events_use_case: Arc::new(RecordPaywallEvents::new(Arc::new(
            PostgresPaywallEventRepository::new(paywall_pool),
        ))),
        token_port,
    });

    activity_routes(state)
}
