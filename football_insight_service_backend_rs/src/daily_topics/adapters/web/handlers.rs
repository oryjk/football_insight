use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, Query, State, rejection::JsonRejection, rejection::QueryRejection},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde_json::{Value, json};
use uuid::Uuid;

use super::dto::{
    CreateTelegraphCommentBody, CreateTelegraphCommentResponseDto, DailyTopicDto,
    DailyTopicsParameters, DailyTopicsResponse, DeleteTelegraphCommentResponseDto,
    EntryInteractionParameters, TelegraphCommentsPageDto, TelegraphCommentsQueryParameters,
    TelegraphReactionResponseDto, TopicDetailParameters, TopicHistoryParameters,
    TopicHistoryResponse,
};
use crate::{
    auth::ports::token_port::TokenPort,
    daily_topics::{
        application::{
            get_daily_topics::GetDailyTopics,
            telegraph_interactions::TelegraphInteractionsService,
        },
        domain::interactions::{
            CreateTelegraphCommentInput, ReactionMutation, TelegraphInteractionError,
            TelegraphReactionKind,
        },
        ports::daily_topics_repository::DailyTopicsRepositoryError,
    },
};

fn extract_optional_user_id(
    token_port: Option<&Extension<Arc<dyn TokenPort>>>,
    headers: &HeaderMap,
) -> Option<Uuid> {
    let port = token_port?;
    let token = headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;
    port.verify_token(token.trim()).ok().map(|claims| claims.sub)
}

fn extract_required_user_id(
    token_port: &dyn TokenPort,
    headers: &HeaderMap,
) -> Result<Uuid, (StatusCode, Json<Value>)> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| interaction_error_response(TelegraphInteractionError::Unauthorized))?;
    token_port
        .verify_token(token)
        .map(|claims| claims.sub)
        .map_err(|_| interaction_error_response(TelegraphInteractionError::Unauthorized))
}

pub(super) async fn get_daily_topics(
    State(use_case): State<Arc<GetDailyTopics>>,
    token_port: Option<Extension<Arc<dyn TokenPort>>>,
    headers: HeaderMap,
    parameters: Result<Query<DailyTopicsParameters>, QueryRejection>,
) -> Result<Json<DailyTopicsResponse>, (StatusCode, &'static str)> {
    let Query(parameters) =
        parameters.map_err(|_| (StatusCode::BAD_REQUEST, "invalid daily topics parameters"))?;
    let query = parameters.try_into().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "team_id must be positive and limit must be between 1 and 20",
        )
    })?;
    let viewer = extract_optional_user_id(token_port.as_ref(), &headers);
    let result = use_case
        .execute_for_viewer(query, viewer)
        .await
        .map_err(|error| {
            tracing::error!(%error, "daily topics request failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "daily topics are temporarily unavailable",
            )
        })?;
    Ok(Json(result.into()))
}

pub(super) async fn get_latest_daily_topic(
    State(use_case): State<Arc<GetDailyTopics>>,
    token_port: Option<Extension<Arc<dyn TokenPort>>>,
    headers: HeaderMap,
    parameters: Result<Query<TopicDetailParameters>, QueryRejection>,
) -> Result<Json<DailyTopicsResponse>, (StatusCode, &'static str)> {
    let Query(parameters) = parameters.map_err(|_| invalid_parameters())?;
    if parameters.team_id <= 0 {
        return Err(invalid_parameters());
    }
    let viewer = extract_optional_user_id(token_port.as_ref(), &headers);
    let result = use_case
        .latest_for_viewer(parameters.team_id, viewer)
        .await
        .map_err(storage_failure)?;
    Ok(Json(result.into()))
}

pub(super) async fn get_daily_topic_by_date(
    State(use_case): State<Arc<GetDailyTopics>>,
    token_port: Option<Extension<Arc<dyn TokenPort>>>,
    headers: HeaderMap,
    Path(date_str): Path<String>,
    parameters: Result<Query<TopicDetailParameters>, QueryRejection>,
) -> Result<Json<DailyTopicsResponse>, (StatusCode, &'static str)> {
    let Query(parameters) = parameters.map_err(|_| invalid_parameters())?;
    if parameters.team_id <= 0 {
        return Err(invalid_parameters());
    }
    let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|_| (StatusCode::BAD_REQUEST, "date must be YYYY-MM-DD"))?;
    let viewer = extract_optional_user_id(token_port.as_ref(), &headers);
    let result = use_case
        .by_date_for_viewer(parameters.team_id, date, viewer)
        .await
        .map_err(storage_failure)?
        .ok_or((StatusCode::NOT_FOUND, "daily roundup not found for date"))?;
    Ok(Json(result.into()))
}

pub(super) async fn get_topic_history(
    State(use_case): State<Arc<GetDailyTopics>>,
    token_port: Option<Extension<Arc<dyn TokenPort>>>,
    headers: HeaderMap,
    parameters: Result<Query<TopicHistoryParameters>, QueryRejection>,
) -> Result<Json<TopicHistoryResponse>, (StatusCode, &'static str)> {
    let Query(parameters) = parameters.map_err(|_| invalid_parameters())?;
    let query = parameters.try_into().map_err(|_| invalid_parameters())?;
    let viewer = extract_optional_user_id(token_port.as_ref(), &headers);
    Ok(Json(
        use_case
            .history_for_viewer(query, viewer)
            .await
            .map_err(storage_failure)?
            .into(),
    ))
}

pub(super) async fn get_topic_detail(
    State(use_case): State<Arc<GetDailyTopics>>,
    token_port: Option<Extension<Arc<dyn TokenPort>>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    parameters: Result<Query<TopicDetailParameters>, QueryRejection>,
) -> Result<Json<DailyTopicDto>, (StatusCode, &'static str)> {
    let Query(parameters) = parameters.map_err(|_| invalid_parameters())?;
    let query = parameters
        .into_query(id)
        .map_err(|_| invalid_parameters())?;
    let viewer = extract_optional_user_id(token_port.as_ref(), &headers);
    let item = use_case
        .detail_for_viewer(query, viewer)
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
    token_port: Option<Extension<Arc<dyn TokenPort>>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    parameters: Result<Query<super::dto::TopicTimelineParameters>, QueryRejection>,
) -> Result<Json<super::dto::TopicTimelineResponse>, (StatusCode, &'static str)> {
    use crate::daily_topics::application::get_daily_topics::TopicTimelineError;
    let Query(parameters) = parameters.map_err(|_| invalid_parameters())?;
    let query = parameters
        .into_query(id)
        .map_err(|_| invalid_parameters())?;
    let viewer = extract_optional_user_id(token_port.as_ref(), &headers);
    Ok(Json(
        use_case
            .timeline_for_viewer(query, viewer)
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

#[derive(Clone)]
pub struct TelegraphInteractionsWebState {
    pub service: Arc<TelegraphInteractionsService>,
    pub token_port: Arc<dyn TokenPort>,
}

pub(super) async fn set_entry_reaction_handler(
    State(state): State<TelegraphInteractionsWebState>,
    headers: HeaderMap,
    Path((topic_id, entry_id)): Path<(String, String)>,
    parameters: Result<Query<EntryInteractionParameters>, QueryRejection>,
    mutation: ReactionMutation,
) -> Result<Json<TelegraphReactionResponseDto>, (StatusCode, Json<Value>)> {
    let user_id = extract_required_user_id(state.token_port.as_ref(), &headers)?;
    let Query(parameters) = parameters
        .map_err(|_| interaction_error_response(TelegraphInteractionError::InvalidInput("参数无效")))?;
    let summary = state
        .service
        .set_reaction(parameters.team_id, &topic_id, &entry_id, user_id, mutation)
        .await
        .map_err(interaction_error_response)?;
    Ok(Json(summary.into()))
}

pub(super) async fn like_entry_handler(
    state: State<TelegraphInteractionsWebState>,
    headers: HeaderMap,
    path: Path<(String, String)>,
    parameters: Result<Query<EntryInteractionParameters>, QueryRejection>,
) -> Result<Json<TelegraphReactionResponseDto>, (StatusCode, Json<Value>)> {
    set_entry_reaction_handler(
        state,
        headers,
        path,
        parameters,
        ReactionMutation::Set(TelegraphReactionKind::Like),
    )
    .await
}

pub(super) async fn unlike_entry_handler(
    state: State<TelegraphInteractionsWebState>,
    headers: HeaderMap,
    path: Path<(String, String)>,
    parameters: Result<Query<EntryInteractionParameters>, QueryRejection>,
) -> Result<Json<TelegraphReactionResponseDto>, (StatusCode, Json<Value>)> {
    set_entry_reaction_handler(
        state,
        headers,
        path,
        parameters,
        ReactionMutation::Clear(TelegraphReactionKind::Like),
    )
    .await
}

pub(super) async fn dislike_entry_handler(
    state: State<TelegraphInteractionsWebState>,
    headers: HeaderMap,
    path: Path<(String, String)>,
    parameters: Result<Query<EntryInteractionParameters>, QueryRejection>,
) -> Result<Json<TelegraphReactionResponseDto>, (StatusCode, Json<Value>)> {
    set_entry_reaction_handler(
        state,
        headers,
        path,
        parameters,
        ReactionMutation::Set(TelegraphReactionKind::Dislike),
    )
    .await
}

pub(super) async fn undislike_entry_handler(
    state: State<TelegraphInteractionsWebState>,
    headers: HeaderMap,
    path: Path<(String, String)>,
    parameters: Result<Query<EntryInteractionParameters>, QueryRejection>,
) -> Result<Json<TelegraphReactionResponseDto>, (StatusCode, Json<Value>)> {
    set_entry_reaction_handler(
        state,
        headers,
        path,
        parameters,
        ReactionMutation::Clear(TelegraphReactionKind::Dislike),
    )
    .await
}

pub(super) async fn list_entry_comments_handler(
    State(state): State<TelegraphInteractionsWebState>,
    headers: HeaderMap,
    Path((topic_id, entry_id)): Path<(String, String)>,
    parameters: Result<Query<TelegraphCommentsQueryParameters>, QueryRejection>,
) -> Result<Json<TelegraphCommentsPageDto>, (StatusCode, Json<Value>)> {
    let Query(parameters) = parameters
        .map_err(|_| interaction_error_response(TelegraphInteractionError::InvalidInput("参数无效")))?;
    let after = parameters
        .decode_cursor(&topic_id, &entry_id)
        .map_err(interaction_error_response)?;
    let viewer = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .and_then(|token| state.token_port.verify_token(token.trim()).ok())
        .map(|claims| claims.sub);
    let page = state
        .service
        .list_comments(
            parameters.team_id,
            &topic_id,
            &entry_id,
            viewer,
            after.as_ref(),
            parameters.limit,
        )
        .await
        .map_err(interaction_error_response)?;
    Ok(Json(TelegraphCommentsPageDto::from_page(
        page,
        parameters.team_id,
        &topic_id,
        &entry_id,
    )))
}

pub(super) async fn create_entry_comment_handler(
    State(state): State<TelegraphInteractionsWebState>,
    headers: HeaderMap,
    Path((topic_id, entry_id)): Path<(String, String)>,
    parameters: Result<Query<EntryInteractionParameters>, QueryRejection>,
    body: Result<Json<CreateTelegraphCommentBody>, JsonRejection>,
) -> Result<(StatusCode, Json<CreateTelegraphCommentResponseDto>), (StatusCode, Json<Value>)> {
    let user_id = extract_required_user_id(state.token_port.as_ref(), &headers)?;
    let Query(parameters) = parameters
        .map_err(|_| interaction_error_response(TelegraphInteractionError::InvalidInput("参数无效")))?;
    let Json(body) = body.map_err(|_| {
        interaction_error_response(TelegraphInteractionError::InvalidInput("请输入评论内容"))
    })?;
    let (comment, comments_count) = state
        .service
        .create_comment(CreateTelegraphCommentInput {
            team_id: parameters.team_id,
            topic_id,
            entry_id,
            user_id,
            content: body.content,
            reply_to_comment_id: body.reply_to_comment_id,
        })
        .await
        .map_err(interaction_error_response)?;
    Ok((
        StatusCode::CREATED,
        Json(CreateTelegraphCommentResponseDto {
            comment: comment.into(),
            comments_count,
        }),
    ))
}

pub(super) async fn delete_entry_comment_handler(
    State(state): State<TelegraphInteractionsWebState>,
    headers: HeaderMap,
    Path((topic_id, entry_id, comment_id)): Path<(String, String, Uuid)>,
    parameters: Result<Query<EntryInteractionParameters>, QueryRejection>,
) -> Result<Json<DeleteTelegraphCommentResponseDto>, (StatusCode, Json<Value>)> {
    let user_id = extract_required_user_id(state.token_port.as_ref(), &headers)?;
    let Query(parameters) = parameters
        .map_err(|_| interaction_error_response(TelegraphInteractionError::InvalidInput("参数无效")))?;
    let comments_count = state
        .service
        .delete_comment(
            parameters.team_id,
            &topic_id,
            &entry_id,
            comment_id,
            user_id,
        )
        .await
        .map_err(interaction_error_response)?;
    Ok(Json(DeleteTelegraphCommentResponseDto { comments_count }))
}

fn interaction_error_response(error: TelegraphInteractionError) -> (StatusCode, Json<Value>) {
    let (status, code) = match error {
        TelegraphInteractionError::TopicOrEntryNotFound => (StatusCode::NOT_FOUND, "NOT_FOUND"),
        TelegraphInteractionError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
        TelegraphInteractionError::ProfileIncomplete => {
            (StatusCode::FORBIDDEN, "PROFILE_INCOMPLETE")
        }
        TelegraphInteractionError::CommentsDisabled => {
            (StatusCode::FORBIDDEN, "COMMENTS_DISABLED")
        }
        TelegraphInteractionError::RateLimited => {
            (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED")
        }
        TelegraphInteractionError::ContentRejected => {
            (StatusCode::BAD_REQUEST, "CONTENT_REJECTED")
        }
        TelegraphInteractionError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "INVALID_INPUT"),
        TelegraphInteractionError::ModerationUnavailable => {
            (StatusCode::SERVICE_UNAVAILABLE, "MODERATION_UNAVAILABLE")
        }
        TelegraphInteractionError::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE"),
    };
    (
        status,
        Json(json!({
            "error": error.to_string(),
            "code": code,
        })),
    )
}
