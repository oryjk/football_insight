use std::sync::Arc;

use axum::{Json, extract::Query, http::StatusCode};

use crate::system_config::{
    adapters::web::dto::{
        MiniProgramReviewConfigDto, MiniProgramReviewConfigQuery, PublicSystemConfigDto,
    },
    application::{
        get_mini_program_review_config::GetMiniProgramReviewConfigUseCase,
        get_public_system_config::GetPublicSystemConfigUseCase,
    },
};

pub async fn get_public_system_config_handler(
    use_case: Arc<GetPublicSystemConfigUseCase>,
) -> Result<Json<PublicSystemConfigDto>, (StatusCode, String)> {
    let config = use_case
        .execute()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

    Ok(Json(config.into()))
}

pub async fn get_mini_program_review_config_handler(
    use_case: Arc<GetMiniProgramReviewConfigUseCase>,
    Query(query): Query<MiniProgramReviewConfigQuery>,
) -> Result<Json<MiniProgramReviewConfigDto>, (StatusCode, String)> {
    let config = use_case
        .execute(query.app_id, query.version)
        .await
        .map_err(|error| {
            let message = error.to_string();
            if message.contains("version is required") {
                return (StatusCode::BAD_REQUEST, "version is required".to_string());
            }
            (StatusCode::INTERNAL_SERVER_ERROR, message)
        })?;

    Ok(Json(config.into()))
}
