use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    admin::{
        adapters::{
            persistence::postgres_admin_user_repository::PostgresAdminUserRepository,
            web::routes::admin_user_routes,
        },
        application::admin_user_service::AdminUserService,
    },
    auth::adapters::security::argon2_password_port::Argon2PasswordPort,
};

pub fn build_admin_routes(pool: PgPool, admin_api_token: Option<String>) -> Router {
    let repository = Arc::new(PostgresAdminUserRepository::new(pool));
    let service = Arc::new(AdminUserService::new(
        repository,
        Arc::new(Argon2PasswordPort),
    ));

    admin_user_routes(service, admin_api_token)
}
