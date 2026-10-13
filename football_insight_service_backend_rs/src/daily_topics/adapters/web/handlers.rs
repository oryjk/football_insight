use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::StatusCode,
};

use super::dto::{
    DailyTopicDto, DailyTopicsParameters, DailyTopicsResponse, TopicDetailParameters,
    TopicHistoryParameters, TopicHistoryResponse,
};
use crate::daily_topics::application::get_daily_topics::GetDailyTopics;
use crate::daily_topics::ports::daily_topics_repository::DailyTopicsRepositoryError;

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

pub(super) async fn get_topic_history(
    State(use_case): State<Arc<GetDailyTopics>>,
    parameters: Result<Query<TopicHistoryParameters>, QueryRejection>,
) -> Result<Json<TopicHistoryResponse>, (StatusCode, &'static str)> {
    let Query(parameters) = parameters.map_err(|_| invalid_parameters())?;
    let query = parameters.try_into().map_err(|_| invalid_parameters())?;
    Ok(Json(
        use_case
            .history(query)
            .await
            .map_err(storage_failure)?
            .into(),
    ))
}

pub(super) async fn get_topic_detail(
    State(use_case): State<Arc<GetDailyTopics>>,
    Path(id): Path<String>,
    parameters: Result<Query<TopicDetailParameters>, QueryRejection>,
) -> Result<Json<DailyTopicDto>, (StatusCode, &'static str)> {
    let Query(parameters) = parameters.map_err(|_| invalid_parameters())?;
    let query = parameters
        .into_query(id)
        .map_err(|_| invalid_parameters())?;
    let item = use_case
        .detail(query)
        .await
        .map_err(storage_failure)?
        .ok_or((StatusCode::NOT_FOUND, "topic not found"))?;
    Ok(Json(item.into()))
}

fn invalid_parameters() -> (StatusCode, &'static str) {
    (StatusCode::BAD_REQUEST, "invalid topic archive parameters")
}

fn storage_failure(error: DailyTopicsRepositoryError) -> (StatusCode, &'static str) {
    tracing::error!(%error, "topic archive request failed");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "topics are temporarily unavailable",
    )
}
