use std::sync::Arc;

use axum::Router;
use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::{
    admin::{
        adapters::{
            persistence::{
                postgres_admin_audit_repository::PostgresAdminAuditRepository,
                postgres_admin_auth_repository::PostgresAdminAuthRepository,
                postgres_admin_user_repository::PostgresAdminUserRepository,
            },
            security::{
                jwt_admin_token_port::JwtAdminTokenPort,
                role_based_admin_authorization::RoleBasedAdminAuthorization,
            },
            web::routes::{admin_audit_routes, admin_auth_routes, admin_user_routes},
        },
        application::{
            admin_audit_service::AdminAuditService, admin_auth_service::AdminAuthService,
            admin_user_service::AdminUserService,
        },
    },
    auth::adapters::security::argon2_password_port::Argon2PasswordPort,
    config::AppConfig,
};

pub fn build_admin_routes(pool: PgPool, config: &AppConfig) -> Router {
    let password_port = Arc::new(Argon2PasswordPort);
    let auth_service = Arc::new(AdminAuthService::new(
        Arc::new(PostgresAdminAuthRepository::new(pool.clone())),
        password_port.clone(),
        Arc::new(JwtAdminTokenPort::new(config.admin_jwt_secret.clone())),
        Arc::new(RoleBasedAdminAuthorization),
        Duration::hours(12),
        Arc::new(Utc::now),
    ));
    let user_service = Arc::new(AdminUserService::new(
        Arc::new(PostgresAdminUserRepository::new(pool.clone())),
        password_port,
    ));

    let audit_service = Arc::new(AdminAuditService::new(Arc::new(
        PostgresAdminAuditRepository::new(pool),
    )));

    admin_auth_routes(auth_service.clone())
        .merge(admin_user_routes(user_service, auth_service.clone()))
        .merge(admin_audit_routes(audit_service, auth_service))
}

pub async fn bootstrap_admin_owner(pool: PgPool, config: &AppConfig) -> anyhow::Result<()> {
    let service = AdminAuthService::new(
        Arc::new(PostgresAdminAuthRepository::new(pool)),
        Arc::new(Argon2PasswordPort),
        Arc::new(JwtAdminTokenPort::new(config.admin_jwt_secret.clone())),
        Arc::new(RoleBasedAdminAuthorization),
        Duration::hours(12),
        Arc::new(Utc::now),
    );
    service
        .ensure_owner(
            config.admin_owner_username.clone(),
            config.admin_owner_password.clone(),
            config.admin_owner_display_name.clone(),
        )
        .await?;
    Ok(())
}
