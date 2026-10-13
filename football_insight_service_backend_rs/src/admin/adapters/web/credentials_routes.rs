use super::credentials_handlers::{AdminCredentialsWebState, update_credentials};
use crate::admin::application::{
    admin_auth_service::AdminAuthService, admin_credentials_service::AdminCredentialsService,
};
use axum::{Router, routing::put};
use std::sync::Arc;
pub fn admin_credentials_routes(
    auth: Arc<AdminAuthService>,
    credentials: Arc<AdminCredentialsService>,
) -> Router {
    Router::new()
        .route("/api/v1/admin/auth/credentials", put(update_credentials))
        .with_state(AdminCredentialsWebState { auth, credentials })
}
