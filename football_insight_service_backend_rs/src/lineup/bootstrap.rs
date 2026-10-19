use super::{
    adapters::{persistence::PostgresSquadRepository, web::routes},
    application::LineupService,
};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_lineup_routes(pool: PgPool) -> Router {
    routes(Arc::new(LineupService::new(Arc::new(
        PostgresSquadRepository::new(pool),
    ))))
}
