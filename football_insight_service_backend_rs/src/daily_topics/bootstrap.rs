use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use super::{
    adapters::{
        persistence::postgres_daily_topics_repository::PostgresDailyTopicsRepository,
        web::routes::{daily_topics_routes_with_auth, telegraph_interactions_routes},
    },
    application::{
        get_daily_topics::GetDailyTopics, telegraph_interactions::TelegraphInteractionsService,
    },
};

pub fn build_daily_topics_routes(
    pool: PgPool,
    token_port: Arc<dyn crate::auth::ports::token_port::TokenPort>,
    content_security: Arc<dyn crate::content_security::ContentSecurityPort>,
) -> Router {
    let repository = Arc::new(PostgresDailyTopicsRepository::new(pool));
    let topics_routes = daily_topics_routes_with_auth(
        Arc::new(GetDailyTopics::new(repository.clone())),
        token_port.clone(),
    );
    let interactions_routes = telegraph_interactions_routes(
        Arc::new(TelegraphInteractionsService::new(
            repository,
            content_security,
        )),
        token_port,
    );
    topics_routes.merge(interactions_routes)
}
