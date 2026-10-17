use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    auth::ports::token_port::TokenPort,
    mini_review::{
        adapters::persistence::postgres_mini_review_repository::PostgresMiniReviewRepository,
        adapters::persistence::postgres_review_exemption::PostgresReviewExemption,
        adapters::web::routes::mini_review_routes_with_exemption,
        ports::mini_review_repository::MiniReviewRepository,
    },
};

pub fn build_mini_review_routes(
    pool: PgPool,
    api_key: Option<String>,
    token_port: Arc<dyn TokenPort>,
    control_user_ids: Vec<uuid::Uuid>,
) -> Router {
    let repository: Arc<dyn MiniReviewRepository> =
        Arc::new(PostgresMiniReviewRepository::new(pool.clone()));
    mini_review_routes_with_exemption(
        repository,
        api_key,
        token_port,
        control_user_ids,
        Arc::new(PostgresReviewExemption::new(pool)),
    )
}
