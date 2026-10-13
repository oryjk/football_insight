use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State, rejection::QueryRejection},
    http::StatusCode,
};

use super::dto::{DailyTopicsParameters, DailyTopicsResponse};
use crate::daily_topics::application::get_daily_topics::GetDailyTopics;

pub(super) async fn get_daily_topics(
    State(use_case): State<Arc<GetDailyTopics>>,
    parameters: Result<Query<DailyTopicsParameters>, QueryRejection>,
) -> Result<Json<DailyTopicsResponse>, (StatusCode, &'static str)> {
    let Query(parameters) =
        parameters.map_err(|_| (StatusCode::BAD_REQUEST, "invalid daily topics parameters"))?;
    let query = parameters.try_into().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "team_id must be positive and limit must be between 1 and 5",
        )
    })?;
    let result = use_case.execute(query).await.map_err(|error| {
        tracing::error!(%error, "daily topics request failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "daily topics are temporarily unavailable",
        )
    })?;
    Ok(Json(result.into()))
}
