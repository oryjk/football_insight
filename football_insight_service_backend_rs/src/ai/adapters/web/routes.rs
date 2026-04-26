use std::sync::Arc;

use axum::{Router, routing::post};

use super::handlers::{AiWebState, ai_chat_handler, ai_chat_stream_handler};

pub fn ai_routes(state: Arc<AiWebState>) -> Router {
    Router::new()
        .route("/api/v1/ai/chat", post(ai_chat_handler))
        .route("/api/v1/ai/chat/stream", post(ai_chat_stream_handler))
        .with_state(state)
}
