use std::sync::Arc;

use axum::{
    Router,
    routing::{get, patch},
};

use crate::admin::{
    adapters::web::handlers::{
        AdminUserWebState, create_admin_user_handler, delete_admin_user_handler,
        list_admin_users_handler, update_admin_user_handler,
    },
    application::admin_user_service::AdminUserService,
};

pub fn admin_user_routes(
    service: Arc<AdminUserService>,
    admin_api_token: Option<String>,
) -> Router {
    let state = AdminUserWebState {
        service,
        admin_api_token,
    };

    Router::new()
        .route(
            "/api/v1/admin/users",
            get(list_admin_users_handler).post(create_admin_user_handler),
        )
        .route(
            "/api/v1/admin/users/{user_id}",
            patch(update_admin_user_handler).delete(delete_admin_user_handler),
        )
        .with_state(state)
}
