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
                postgres_admin_brand_repository::PostgresAdminBrandRepository,
                postgres_admin_credentials_repository::PostgresAdminCredentialsRepository,
                postgres_admin_payment_repository::PostgresAdminPaymentRepository,
                postgres_admin_paywall_repository::PostgresAdminPaywallRepository,
                postgres_admin_product_repository::PostgresAdminProductRepository,
                postgres_admin_review_exempt_repository::PostgresAdminReviewExemptRepository,
                postgres_admin_support_bases_repository::PostgresAdminSupportBasesRepository,
                postgres_admin_topic_readers_repository::PostgresAdminTopicReadersRepository,
                postgres_admin_user_repository::PostgresAdminUserRepository,
            },
            security::{
                jwt_admin_token_port::JwtAdminTokenPort,
                role_based_admin_authorization::RoleBasedAdminAuthorization,
            },
            web::{
                brand_routes::admin_brand_routes,
                credentials_routes::admin_credentials_routes,
                payment_routes::admin_payment_routes,
                paywall_routes::admin_paywall_routes,
                product_routes::admin_product_routes,
                routes::{admin_audit_routes, admin_auth_routes, admin_user_routes},
                setup_routes::admin_setup_routes,
                review_exempt_routes::admin_review_exempt_routes,
                support_bases_routes::admin_support_bases_routes,
                topic_readers_routes::admin_topic_readers_routes,
            },
        },
        application::{
            admin_audit_service::AdminAuditService, admin_auth_service::AdminAuthService,
            admin_brand_service::AdminBrandService,
            admin_credentials_service::AdminCredentialsService,
            admin_payment_service::AdminPaymentService,
            admin_paywall_service::AdminPaywallService, admin_product_service::AdminProductService,
            admin_setup_service::AdminSetupService,
            admin_review_exempt_service::AdminReviewExemptService,
            admin_support_bases_service::AdminSupportBasesService,
            admin_topic_readers_service::AdminTopicReadersService,
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
    let setup_service = Arc::new(AdminSetupService::new(
        Arc::new(PostgresAdminAuthRepository::new(pool.clone())),
        password_port.clone(),
    ));
    let user_service = Arc::new(AdminUserService::new(
        Arc::new(PostgresAdminUserRepository::new(pool.clone())),
        password_port.clone(),
    ));
    let credentials_service = Arc::new(AdminCredentialsService::new(
        Arc::new(PostgresAdminCredentialsRepository::new(pool.clone())),
        password_port,
    ));

    let product_service = Arc::new(AdminProductService::new(Arc::new(
        PostgresAdminProductRepository::new(pool.clone()),
    )));

    let payment_service = Arc::new(AdminPaymentService::new(Arc::new(
        PostgresAdminPaymentRepository::new(pool.clone()),
    )));

    let paywall_service = Arc::new(AdminPaywallService::new(Arc::new(
        PostgresAdminPaywallRepository::new(pool.clone()),
    )));

    let topic_readers_service = Arc::new(AdminTopicReadersService::new(Arc::new(
        PostgresAdminTopicReadersRepository::new(pool.clone()),
    )));

    let support_bases_service = Arc::new(AdminSupportBasesService::new(Arc::new(
        PostgresAdminSupportBasesRepository::new(pool.clone()),
    )));

    let review_exempt_service = Arc::new(AdminReviewExemptService::new(Arc::new(
        PostgresAdminReviewExemptRepository::new(pool.clone()),
    )));

    let brand_service = Arc::new(AdminBrandService::new(Arc::new(
        PostgresAdminBrandRepository::new(pool.clone()),
    )));

    let audit_service = Arc::new(AdminAuditService::new(Arc::new(
        PostgresAdminAuditRepository::new(pool),
    )));

    admin_auth_routes(auth_service.clone())
        .merge(admin_setup_routes(setup_service))
        .merge(admin_credentials_routes(
            auth_service.clone(),
            credentials_service,
        ))
        .merge(admin_user_routes(user_service, auth_service.clone()))
        .merge(admin_product_routes(product_service, auth_service.clone()))
        .merge(admin_payment_routes(payment_service, auth_service.clone()))
        .merge(admin_paywall_routes(paywall_service, auth_service.clone()))
        .merge(admin_support_bases_routes(
            support_bases_service,
            auth_service.clone(),
        ))
        .merge(admin_review_exempt_routes(
            review_exempt_service,
            auth_service.clone(),
        ))
        .merge(admin_brand_routes(brand_service, auth_service.clone()))
        .merge(admin_topic_readers_routes(
            topic_readers_service,
            auth_service.clone(),
        ))
        .merge(admin_audit_routes(audit_service, auth_service))
}

pub async fn bootstrap_admin_owner(pool: PgPool, config: &AppConfig) -> anyhow::Result<()> {
    if config.admin_owner_password.is_empty() {
        return Ok(());
    }
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
