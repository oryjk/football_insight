use std::{sync::Arc, time::Duration};

use axum::Router;

use crate::{
    config::AppConfig,
    insight::ports::insight_query_repository::InsightQueryRepository,
    mcp::{
        adapters::{
            protocol::McpProtocol,
            web::{McpWebState, routes},
            wechat_promo_code::WechatPromoCodes,
        },
        application::McpQueryService,
    },
};

const QUERY_CACHE_TTL: Duration = Duration::from_secs(30);

pub fn build_mcp_routes(repository: Arc<dyn InsightQueryRepository>, config: &AppConfig) -> Router {
    let service = McpQueryService::new(repository, config.public_base_url.clone(), QUERY_CACHE_TTL);
    routes(McpWebState {
        protocol: Arc::new(McpProtocol::new(service, &config.public_base_url)),
        codes: Arc::new(WechatPromoCodes::new(
            &config.wechat_mini_app_id,
            &config.wechat_mini_app_secret,
        )),
    })
}
