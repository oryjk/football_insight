use std::sync::Arc;

use axum::{Router, routing::post};

use crate::auth_license::adapters::web::handlers::*;

pub fn auth_license_routes(state: Arc<AuthLicenseWebState>) -> Router {
    Router::new()
        .route(
            "/api/v1/auth/generate-license",
            post(generate_license_handler),
        )
        .route("/api/v1/auth/bind-license", post(bind_license_handler))
        .with_state(state)
}
