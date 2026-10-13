use super::setup_handlers::{initialize_admin, no_store, setup_status};
use crate::admin::application::admin_setup_service::AdminSetupService;
use axum::{Router, middleware::map_response, routing::get};
use std::sync::Arc;
pub fn admin_setup_routes(service: Arc<AdminSetupService>) -> Router {
    Router::new()
        .route(
            "/api/v1/admin/auth/setup",
            get(setup_status).post(initialize_admin),
        )
        .with_state(service)
        .layer(map_response(no_store))
}
