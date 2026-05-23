use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    insight::ports::insight_query_repository::InsightQueryRepository,
    system_config::{
        adapters::{
            persistence::{
                postgres_mini_program_review_config_port::PostgresMiniProgramReviewConfigPort,
                postgres_system_config_port::PostgresSystemConfigPort,
            },
            web::routes::system_config_routes,
        },
        application::{
            get_mini_program_review_config::GetMiniProgramReviewConfigUseCase,
            get_public_system_config::GetPublicSystemConfigUseCase,
        },
        ports::system_config_port::SystemConfigPort,
    },
};

pub struct SystemConfigBootstrap {
    pub routes: Router,
    pub system_config_port: Arc<dyn SystemConfigPort>,
}

pub fn build_system_config_routes(
    pool: PgPool,
    insight_repository: Arc<dyn InsightQueryRepository>,
) -> SystemConfigBootstrap {
    let system_config_port: Arc<dyn SystemConfigPort> =
        Arc::new(PostgresSystemConfigPort::new(pool.clone()));
    let public_config_use_case = Arc::new(GetPublicSystemConfigUseCase::new(
        system_config_port.clone(),
        insight_repository,
    ));
    let mini_program_review_config_use_case = Arc::new(GetMiniProgramReviewConfigUseCase::new(
        Arc::new(PostgresMiniProgramReviewConfigPort::new(pool)),
    ));

    SystemConfigBootstrap {
        routes: system_config_routes(public_config_use_case, mini_program_review_config_use_case),
        system_config_port,
    }
}
