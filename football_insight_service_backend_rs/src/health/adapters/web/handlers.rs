use std::sync::Arc;

use axum::{Json, http::StatusCode};

use crate::health::{
    application::get_health::GetHealthUseCase, domain::health_status::HealthStatus,
};

pub async fn get_health_handler(
    use_case: Arc<GetHealthUseCase>,
) -> Result<Json<HealthStatus>, (StatusCode, String)> {
    let status = use_case
        .execute()
        .await
        .map_err(|error| (StatusCode::SERVICE_UNAVAILABLE, error.to_string()))?;

    Ok(Json(status))
}
