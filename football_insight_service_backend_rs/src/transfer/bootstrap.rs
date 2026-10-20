use super::{
    adapters::{
        persistence::PostgresTransferRepository,
        web::{TransferWebState, transfer_routes},
    },
    application::TransferService,
};
use crate::{
    auth::ports::token_port::TokenPort, config::AppConfig,
    share_code::adapters::wechat_code::WechatTransferCode,
};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_transfer_routes(
    pool: PgPool,
    token_port: Arc<dyn TokenPort>,
    config: &AppConfig,
) -> Router {
    let store = Arc::new(PostgresTransferRepository::new(pool));
    let codes = Arc::new(WechatTransferCode::new(
        config.wechat_mini_app_id.clone(),
        config.wechat_mini_app_secret.clone(),
    ));
    let service = TransferService::new(
        store.clone(),
        store.clone(),
        store,
        codes,
        Arc::new(chrono::Utc::now),
    );
    transfer_routes(Arc::new(TransferWebState {
        service: Arc::new(service),
        token_port,
    }))
}
