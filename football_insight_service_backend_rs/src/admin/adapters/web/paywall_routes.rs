use super::{
    paywall_handlers::{self, AdminPaywallWebState},
    product_handlers::product_no_store,
};
use crate::admin::application::{
    admin_auth_service::AdminAuthService, admin_paywall_service::AdminPaywallService,
};
use axum::{Router, routing::get};
use std::sync::Arc;

pub fn admin_paywall_routes(
    service: Arc<AdminPaywallService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    Router::new()
        .route(
            "/api/v1/admin/paywall/funnel",
            get(paywall_handlers::funnel),
        )
        .route(
            "/api/v1/admin/paywall/by-tier",
            get(paywall_handlers::by_tier),
        )
        .with_state(AdminPaywallWebState {
            service,
            auth_service,
        })
        .layer(axum::middleware::map_response(product_no_store))
}
