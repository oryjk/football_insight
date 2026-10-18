use super::{
    adapters::{
        persistence::PostgresRecapRepository,
        web::{WebState, routes},
        wechat_code::WechatRecapCode,
    },
    application::RecapService,
};
use crate::{auth::ports::token_port::TokenPort, config::AppConfig};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
pub fn build_season_recap_routes(
    pool: PgPool,
    token: Arc<dyn TokenPort>,
    config: &AppConfig,
) -> Router {
    routes(WebState {
        service: Arc::new(RecapService::new(
            Arc::new(PostgresRecapRepository::new(pool)),
            Arc::new(WechatRecapCode::new(
                config.wechat_mini_app_id.clone(),
                config.wechat_mini_app_secret.clone(),
            )),
        )),
        token,
    })
}
