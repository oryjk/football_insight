use super::{
    adapters::{
        my_web::{MyLineupWebState, my_routes},
        persistence::{PostgresLineupRepository, PostgresSquadRepository},
        public_persistence::PostgresPublicLineupRepository,
        public_web::{PublicLineupWebState, public_routes},
        web::routes,
    },
    application::{LineupService, my_lineup::MyLineupService, public_lineup::PublicLineupService},
};
use crate::auth::ports::token_port::TokenPort;
use crate::{config::AppConfig, share_code::adapters::wechat_code::WechatLineupCode};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_lineup_routes(
    pool: PgPool,
    token_port: Arc<dyn TokenPort>,
    config: &AppConfig,
) -> Router {
    let squads = Arc::new(PostgresSquadRepository::new(pool.clone()));
    let lineups = Arc::new(PostgresLineupRepository::new(pool.clone()));
    let public = public_routes(Arc::new(PublicLineupWebState {
        service: Arc::new(PublicLineupService::new(
            squads.clone(),
            Arc::new(PostgresPublicLineupRepository::new(pool)),
            lineups.clone(),
            Arc::new(WechatLineupCode::new(
                config.wechat_mini_app_id.clone(),
                config.wechat_mini_app_secret.clone(),
            )),
        )),
        token_port: token_port.clone(),
    }));
    routes(Arc::new(LineupService::new(squads.clone())))
        .merge(public)
        .merge(my_routes(Arc::new(MyLineupWebState {
            service: Arc::new(MyLineupService::new(squads, lineups.clone(), lineups)),
            token_port,
        })))
}
