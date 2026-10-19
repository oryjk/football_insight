use super::{
    adapters::{
        my_web::{MyLineupWebState, my_routes},
        persistence::{PostgresLineupRepository, PostgresSquadRepository},
        web::routes,
    },
    application::{LineupService, my_lineup::MyLineupService},
};
use crate::auth::ports::token_port::TokenPort;
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_lineup_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router {
    let squads = Arc::new(PostgresSquadRepository::new(pool.clone()));
    let lineups = Arc::new(PostgresLineupRepository::new(pool));
    routes(Arc::new(LineupService::new(squads.clone()))).merge(my_routes(Arc::new(
        MyLineupWebState {
            service: Arc::new(MyLineupService::new(squads, lineups)),
            token_port,
        },
    )))
}
