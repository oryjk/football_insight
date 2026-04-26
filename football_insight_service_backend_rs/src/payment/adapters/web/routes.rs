use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use super::handlers::{
    PaymentWebState, create_membership_order_handler, get_membership_product_handler,
    get_order_status_handler, wechat_notify_handler,
};

pub fn payment_routes(state: Arc<PaymentWebState>) -> Router {
    Router::new()
        .route(
            "/api/v1/payment/membership-product",
            get(get_membership_product_handler),
        )
        .route(
            "/api/v1/payment/membership/order",
            post(create_membership_order_handler),
        )
        .route(
            "/api/v1/payment/order/{order_no}",
            get(get_order_status_handler),
        )
        .route("/api/v1/payment/wx-notify", post(wechat_notify_handler))
        .with_state(state)
}
