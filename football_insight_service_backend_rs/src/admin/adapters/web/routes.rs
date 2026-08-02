use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::admin::{
    adapters::web::handlers::{
        AdminAuditWebState, AdminUserWebState, adjust_admin_user_membership_handler,
        admin_login_handler, admin_logout_handler, admin_me_handler, create_admin_user_handler,
        delete_admin_user_handler, disable_admin_user_handler, get_admin_user_handler,
        list_admin_audit_logs_handler, list_admin_users_handler, restore_admin_user_handler,
        update_admin_user_handler,
    },
    application::{
        admin_audit_service::AdminAuditService, admin_auth_service::AdminAuthService,
        admin_user_service::AdminUserService,
    },
};

pub fn admin_auth_routes(service: Arc<AdminAuthService>) -> Router {
    Router::new()
        .route("/api/v1/admin/auth/login", post(admin_login_handler))
        .route("/api/v1/admin/auth/me", get(admin_me_handler))
        .route("/api/v1/admin/auth/logout", post(admin_logout_handler))
        .with_state(service)
}

pub fn admin_audit_routes(
    service: Arc<AdminAuditService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    Router::new()
        .route(
            "/api/v1/admin/audit-logs",
            get(list_admin_audit_logs_handler),
        )
        .with_state(AdminAuditWebState {
            service,
            auth_service,
        })
}

pub fn admin_user_routes(
    service: Arc<AdminUserService>,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    let state = AdminUserWebState {
        service,
        auth_service,
    };

    Router::new()
        .route(
            "/api/v1/admin/users",
            get(list_admin_users_handler).post(create_admin_user_handler),
        )
        .route(
            "/api/v1/admin/users/{user_id}",
            get(get_admin_user_handler)
                .patch(update_admin_user_handler)
                .delete(delete_admin_user_handler),
        )
        .route(
            "/api/v1/admin/users/{user_id}/disable",
            post(disable_admin_user_handler),
        )
        .route(
            "/api/v1/admin/users/{user_id}/restore",
            post(restore_admin_user_handler),
        )
        .route(
            "/api/v1/admin/users/{user_id}/membership",
            post(adjust_admin_user_membership_handler),
        )
        .with_state(state)
}
