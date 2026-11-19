use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{auth::ports::token_port::TokenPort, content_security::ContentSecurityPort};

use super::{
    adapters::{
        persistence::PostgresDirectorRepository,
        web::{DirectorWebState, director_routes},
    },
    application::DirectorService,
};

pub fn build_director_routes(
    pool: PgPool,
    token_port: Arc<dyn TokenPort>,
    content_security: Arc<dyn ContentSecurityPort>,
) -> Router {
    let repository = Arc::new(PostgresDirectorRepository::new(pool));
    let service = Arc::new(DirectorService::new(
        repository,
        content_security,
        Arc::new(chrono::Utc::now),
    ));
    director_routes(Arc::new(DirectorWebState {
        service,
        token_port,
    }))
}
