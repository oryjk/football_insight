use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::token_port::TokenPort,
    auth_license::{
        adapters::{
            persistence::postgres_license_repository::PostgresLicenseRepository,
            web::{handlers::AuthLicenseWebState, routes::auth_license_routes},
        },
        application::{bind_license::BindLicenseUseCase, generate_license::GenerateLicenseUseCase},
    },
};

pub fn build_auth_license_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router {
    let repository = Arc::new(PostgresLicenseRepository::new(pool));
    let state = Arc::new(AuthLicenseWebState {
        generate_license_use_case: Arc::new(GenerateLicenseUseCase::new(repository.clone())),
        bind_license_use_case: Arc::new(BindLicenseUseCase::new(repository)),
        token_port,
    });

    auth_license_routes(state)
}
