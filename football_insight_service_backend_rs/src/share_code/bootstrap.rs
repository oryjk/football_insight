use super::{
    adapters::{
        web::{WebState, routes},
        wechat_code::WechatTicketWatchCode,
    },
    application::ShareCodeService,
};
use crate::{auth::ports::token_port::TokenPort, config::AppConfig};
use axum::Router;
use std::sync::Arc;

pub fn build_share_code_routes(token: Arc<dyn TokenPort>, config: &AppConfig) -> Router {
    routes(WebState {
        service: Arc::new(ShareCodeService::new(Arc::new(WechatTicketWatchCode::new(
            config.wechat_mini_app_id.clone(),
            config.wechat_mini_app_secret.clone(),
        )))),
        token,
    })
}
