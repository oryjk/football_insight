use super::{
    adapters::{persistence::PostgresOutlookRepository, web::routes},
    application::OutlookService,
};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
pub fn build_match_outlook_routes(pool: PgPool) -> Router {
    routes(Arc::new(OutlookService::new(Arc::new(
        PostgresOutlookRepository::new(pool),
    ))))
}
