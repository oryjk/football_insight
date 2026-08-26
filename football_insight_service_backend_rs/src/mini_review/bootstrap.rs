use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::mini_review::{
    adapters::persistence::postgres_mini_review_repository::PostgresMiniReviewRepository,
    adapters::web::routes::mini_review_routes,
    ports::mini_review_repository::MiniReviewRepository,
};

pub fn build_mini_review_routes(pool: PgPool, api_key: Option<String>) -> Router {
    let repository: Arc<dyn MiniReviewRepository> =
        Arc::new(PostgresMiniReviewRepository::new(pool));
    mini_review_routes(repository, api_key)
}
