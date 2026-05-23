use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::health::{
    adapters::{persistence::postgres_health_port::PostgresHealthPort, web::routes::health_routes},
    application::get_health::GetHealthUseCase,
};

pub fn build_health_routes(pool: PgPool) -> Router {
    health_routes(Arc::new(GetHealthUseCase::new(Arc::new(
        PostgresHealthPort::new(pool),
    ))))
}
