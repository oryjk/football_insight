use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use super::{
    adapters::{
        persistence::postgres_daily_topics_repository::PostgresDailyTopicsRepository,
        web::routes::daily_topics_routes,
    },
    application::get_daily_topics::GetDailyTopics,
};

pub fn build_daily_topics_routes(pool: PgPool) -> Router {
    let repository = Arc::new(PostgresDailyTopicsRepository::new(pool));
    daily_topics_routes(Arc::new(GetDailyTopics::new(repository)))
}
