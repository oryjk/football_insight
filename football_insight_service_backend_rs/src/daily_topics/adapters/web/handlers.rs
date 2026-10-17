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

pub(super) async fn get_topic_timeline(
    State(use_case): State<Arc<GetDailyTopics>>,
    Path(id): Path<String>,
    parameters: Result<Query<super::dto::TopicTimelineParameters>, QueryRejection>,
) -> Result<Json<super::dto::TopicTimelineResponse>, (StatusCode, &'static str)> {
    use crate::daily_topics::application::get_daily_topics::TopicTimelineError;
    let Query(parameters) = parameters.map_err(|_| invalid_parameters())?;
    let query = parameters
        .into_query(id)
        .map_err(|_| invalid_parameters())?;
    Ok(Json(
        use_case
            .timeline(query)
            .await
            .map_err(|error| match error {
                TopicTimelineError::AnchorNotFound => (StatusCode::NOT_FOUND, "topic not found"),
                TopicTimelineError::UnrelatedTag => (
                    StatusCode::BAD_REQUEST,
                    "club tag does not belong to this topic",
                ),
                TopicTimelineError::Storage(error) => storage_failure(error),
            })?
            .into(),
    ))
}

fn storage_failure(error: DailyTopicsRepositoryError) -> (StatusCode, &'static str) {
    tracing::error!(%error, "topic archive request failed");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "topics are temporarily unavailable",
    )
}

pub(super) async fn record_topic_visit(
    State(use_case): State<Arc<GetDailyTopics>>,
    token_port: Option<axum::Extension<Arc<dyn crate::auth::ports::token_port::TokenPort>>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    parameters: Result<Query<TopicDetailParameters>, QueryRejection>,
    body: Result<Json<super::dto::TopicVisitBody>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<super::dto::TopicVisitResponse>, (StatusCode, &'static str)> {
    use sha2::{Digest, Sha256};
    let Query(parameters) = parameters.map_err(|_| invalid_parameters())?;
    let Json(body) = body.map_err(|_| invalid_parameters())?;
    let visitor = uuid::Uuid::parse_str(&body.visitor_id).map_err(|_| invalid_parameters())?;
    if visitor.is_nil() {
        return Err(invalid_parameters());
    }
    let query = parameters
        .into_query(id)
        .map_err(|_| invalid_parameters())?;
    let hash = format!("{:x}", Sha256::digest(visitor.as_bytes()));
    // A missing or invalid token is not an error: the visit simply stays anonymous.
    let user_id = token_port.and_then(|port| {
        let token = headers
            .get(axum::http::header::AUTHORIZATION)?
            .to_str()
            .ok()?
            .strip_prefix("Bearer ")?;
        port.verify_token(token).ok().map(|claims| claims.sub)
    });
    let visit = crate::daily_topics::domain::heat::TopicVisit::new(query, hash)
        .map_err(|_| invalid_parameters())?
        .with_user(user_id);
    let heat = use_case
        .record_visit(visit)
        .await
        .map_err(storage_failure)?
        .ok_or((StatusCode::NOT_FOUND, "topic not found"))?;
    Ok(Json(super::dto::TopicVisitResponse { heat }))
}
