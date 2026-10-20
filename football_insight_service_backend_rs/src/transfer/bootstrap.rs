use super::{
    adapters::{
        persistence::PostgresTransferRepository,
        web::{TransferWebState, transfer_routes},
    },
    application::TransferService,
};
use crate::auth::ports::token_port::TokenPort;
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;

pub fn build_transfer_routes(pool: PgPool, token_port: Arc<dyn TokenPort>) -> Router {
    let store = Arc::new(PostgresTransferRepository::new(pool));
    let service = TransferService::new(
        store.clone(),
        store.clone(),
        store,
        Arc::new(chrono::Utc::now),
    );
    transfer_routes(Arc::new(TransferWebState {
        service: Arc::new(service),
        token_port,
    }))
}
