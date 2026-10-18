use super::{
    brand_handlers::{self, AdminBrandWebState},
    product_handlers::product_no_store,
};
use crate::admin::application::{
    admin_auth_service::AdminAuthService, admin_brand_service::AdminBrandService,
};
use axum::{Router, routing::get};
use std::sync::Arc;

pub fn admin_brand_routes(
    service: Arc<AdminBrandService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    Router::new()
        .route(
            "/api/v1/admin/brand/subtitle",
            get(brand_handlers::get_subtitle).put(brand_handlers::set_subtitle),
        )
        .with_state(AdminBrandWebState { service, auth_service })
        .layer(axum::middleware::map_response(product_no_store))
}
