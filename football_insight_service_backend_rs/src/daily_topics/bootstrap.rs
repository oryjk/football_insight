use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use super::{
    adapters::{
        persistence::postgres_daily_topics_repository::PostgresDailyTopicsRepository,
        web::routes::daily_topics_routes_with_auth,
    },
    application::get_daily_topics::GetDailyTopics,
};

pub fn build_daily_topics_routes(
    pool: PgPool,
    token_port: Arc<dyn crate::auth::ports::token_port::TokenPort>,
) -> Router {
    let repository = Arc::new(PostgresDailyTopicsRepository::new(pool));
    daily_topics_routes_with_auth(Arc::new(GetDailyTopics::new(repository)), token_port)
}
