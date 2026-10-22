use super::{
    product_handlers::product_no_store,
    review_exempt_handlers::{self, AdminReviewExemptWebState},
};
use crate::admin::application::{
    admin_auth_service::AdminAuthService, admin_review_exempt_service::AdminReviewExemptService,
};
use axum::{
    Router,
    routing::{get, put},
};
use std::sync::Arc;

pub fn admin_review_exempt_routes(
    service: Arc<AdminReviewExemptService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    Router::new()
        .route(
            "/api/v1/admin/review-exempt-users",
            get(review_exempt_handlers::list_exempt_users),
        )
        .route(
            "/api/v1/admin/review-exempt-users/{user_id}",
            put(review_exempt_handlers::add_exempt_user)
                .delete(review_exempt_handlers::remove_exempt_user),
        )
        .with_state(AdminReviewExemptWebState {
            service,
            auth_service,
        })
        .layer(axum::middleware::map_response(product_no_store))
}
