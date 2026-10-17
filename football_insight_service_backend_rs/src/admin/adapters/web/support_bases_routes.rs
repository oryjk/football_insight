use super::{
    product_handlers::product_no_store,
    support_bases_handlers::{self, AdminSupportBasesWebState},
};
use crate::admin::application::{
    admin_auth_service::AdminAuthService, admin_support_bases_service::AdminSupportBasesService,
};
use axum::{Router, routing::{get, put}};
use std::sync::Arc;

pub fn admin_support_bases_routes(
    service: Arc<AdminSupportBasesService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    Router::new()
        .route("/api/v1/admin/support/matches", get(support_bases_handlers::list_matches))
        .route(
            "/api/v1/admin/support/matches/{match_id}/bases",
            put(support_bases_handlers::set_bases),
        )
        .with_state(AdminSupportBasesWebState { service, auth_service })
        .layer(axum::middleware::map_response(product_no_store))
}
