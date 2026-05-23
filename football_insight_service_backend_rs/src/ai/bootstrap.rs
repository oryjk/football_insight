use std::sync::Arc;

use axum::Router;

use crate::{
    ai::{
        adapters::{
            integration::rig_openai_chat_port::{DisabledAiChatPort, RigOpenAiChatPort},
            web::{handlers::AiWebState, routes::ai_routes},
        },
        application::chat_with_model::ChatWithModelUseCase,
        ports::ai_chat_port::AiChatPort,
    },
    auth::application::get_current_user::GetCurrentUserUseCase,
    config::AppConfig,
    insight::ports::insight_query_repository::InsightQueryRepository,
    system_config::ports::system_config_port::SystemConfigPort,
};

pub fn build_ai_routes(
    config: &AppConfig,
    system_config_port: Arc<dyn SystemConfigPort>,
    insight_repository: Arc<dyn InsightQueryRepository>,
    get_current_user_use_case: Arc<GetCurrentUserUseCase>,
) -> Router {
    let ai_chat_port: Arc<dyn AiChatPort> = match config.openai_api_key.clone() {
        Some(openai_api_key) => Arc::new(RigOpenAiChatPort::new(
            openai_api_key,
            config.ai_chat_model.clone(),
            config.openai_base_url.clone(),
            system_config_port,
            insight_repository,
        )),
        None => Arc::new(DisabledAiChatPort::new()),
    };
    let state = Arc::new(AiWebState {
        chat_with_model_use_case: Arc::new(ChatWithModelUseCase::new(ai_chat_port)),
        get_current_user_use_case,
    });

    ai_routes(state)
}
