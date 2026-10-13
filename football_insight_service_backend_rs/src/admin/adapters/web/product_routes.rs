use super::product_handlers::{
    AdminProductWebState, create_product, delete_product, get_product, list_products,
    update_product,
};
use crate::admin::application::{
    admin_auth_service::AdminAuthService, admin_product_service::AdminProductService,
};
use axum::{Router, routing::get};
use std::sync::Arc;
pub fn admin_product_routes(
    service: Arc<AdminProductService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    Router::new()
        .route(
            "/api/v1/admin/products",
            get(list_products).post(create_product),
        )
        .route(
            "/api/v1/admin/products/{product_key}",
            get(get_product).put(update_product).delete(delete_product),
        )
        .with_state(AdminProductWebState {
            service,
            auth_service,
        })
}
