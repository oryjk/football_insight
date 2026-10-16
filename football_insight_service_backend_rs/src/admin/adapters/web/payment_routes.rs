use super::{
    payment_handlers::{self, AdminPaymentWebState},
    product_handlers::product_no_store,
};
use crate::admin::application::{
    admin_auth_service::AdminAuthService, admin_payment_service::AdminPaymentService,
};
use axum::{Router, routing::get};
use std::sync::Arc;

pub fn admin_payment_routes(
    service: Arc<AdminPaymentService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    Router::new()
        .route(
            "/api/v1/admin/payment/funnel",
            get(payment_handlers::funnel),
        )
        .route(
            "/api/v1/admin/payment/errors",
            get(payment_handlers::errors),
        )
        .route(
            "/api/v1/admin/payment/orders/{order_no}/events",
            get(payment_handlers::timeline),
        )
        .route(
            "/api/v1/admin/payment/stuck-orders",
            get(payment_handlers::stuck),
        )
        .with_state(AdminPaymentWebState {
            service,
            auth_service,
        })
        .layer(axum::middleware::map_response(product_no_store))
}
