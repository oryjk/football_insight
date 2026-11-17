use std::sync::Arc;

use axum::{
    Router,
    routing::{get, put},
};

use super::{
    product_handlers::product_no_store,
    telegraph_comments_handlers::{self, AdminTelegraphCommentsWebState},
};
use crate::admin::application::{
    admin_auth_service::AdminAuthService,
    admin_telegraph_comments_service::AdminTelegraphCommentsService,
};

pub fn admin_telegraph_comments_routes(
    service: Arc<AdminTelegraphCommentsService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    Router::new()
        .route(
            "/api/v1/admin/telegraph-comments",
            get(telegraph_comments_handlers::list_comments),
        )
        .route(
            "/api/v1/admin/telegraph-comments/config",
            put(telegraph_comments_handlers::set_comments_config),
        )
        .route(
            "/api/v1/admin/telegraph-comments/{comment_id}/status",
            put(telegraph_comments_handlers::set_comment_status),
        )
        .with_state(AdminTelegraphCommentsWebState {
            service,
            auth_service,
        })
        .layer(axum::middleware::map_response(product_no_store))
}
