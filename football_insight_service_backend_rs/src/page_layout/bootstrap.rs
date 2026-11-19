use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    admin::application::admin_auth_service::AdminAuthService,
    config::AppConfig,
    page_layout::{
        adapters::{
            minio_storage::MinioLayoutImageStoragePort,
            persistence::PostgresPageLayoutRepository,
            web::{self, AdminPageLayoutWebState, PublicPageLayoutWebState},
        },
        application::PageLayoutService,
    },
};

pub fn build_page_layout_routes(pool: PgPool, config: &AppConfig) -> Router {
    let service = Arc::new(PageLayoutService::new(
        Arc::new(PostgresPageLayoutRepository::new(pool)),
        Arc::new(MinioLayoutImageStoragePort::new(config.minio.clone())),
    ));
    web::public_routes(PublicPageLayoutWebState { service })
}

pub fn build_admin_page_layout_routes(
    pool: PgPool,
    config: &AppConfig,
    auth_service: Arc<AdminAuthService>,
) -> Router {
    let service = Arc::new(PageLayoutService::new(
        Arc::new(PostgresPageLayoutRepository::new(pool)),
        Arc::new(MinioLayoutImageStoragePort::new(config.minio.clone())),
    ));
    web::admin_routes(AdminPageLayoutWebState {
        service,
        auth_service,
    })
}
