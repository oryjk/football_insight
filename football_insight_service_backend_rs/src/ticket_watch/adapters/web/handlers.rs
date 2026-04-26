use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::ports::token_port::TokenPort;
use crate::ticket_watch::{
    adapters::web::dto::{
        TicketWatchBlockInterestDto, TicketWatchCurrentBoardDto, TicketWatchCurrentMatchDto,
        TicketWatchInventoryEntryDto, TicketWatchMatchSummaryDto, TicketWatchRegionDto,
        TicketWatchTrackedInterestDto,
        ToggleTicketWatchBlockInterestRequest,
    },
    application::{
        get_current_ticket_watch_board::GetCurrentTicketWatchBoardUseCase,
        get_current_ticket_watch_match::GetCurrentTicketWatchMatchUseCase,
        get_match_block_interests::GetMatchBlockInterestsUseCase,
        get_match_ticket_inventory::GetMatchTicketInventoryUseCase,
        get_match_tracked_interests::GetMatchTrackedInterestsUseCase,
        list_ticket_watch_matches::ListTicketWatchMatchesUseCase,
        list_ticket_watch_regions::ListTicketWatchRegionsUseCase,
        toggle_match_block_interest::ToggleMatchBlockInterestUseCase,
    },
};

#[derive(Clone)]
pub struct TicketWatchWebState {
    pub get_current_ticket_watch_board_use_case: Arc<GetCurrentTicketWatchBoardUseCase>,
    pub get_current_ticket_watch_match_use_case: Arc<GetCurrentTicketWatchMatchUseCase>,
    pub list_ticket_watch_matches_use_case: Arc<ListTicketWatchMatchesUseCase>,
    pub list_ticket_watch_regions_use_case: Arc<ListTicketWatchRegionsUseCase>,
    pub get_match_ticket_inventory_use_case: Arc<GetMatchTicketInventoryUseCase>,
    pub get_match_block_interests_use_case: Arc<GetMatchBlockInterestsUseCase>,
    pub get_match_tracked_interests_use_case: Arc<GetMatchTrackedInterestsUseCase>,
    pub toggle_match_block_interest_use_case: Arc<ToggleMatchBlockInterestUseCase>,
    pub token_port: Arc<dyn TokenPort>,
}

#[derive(Debug, Deserialize)]
pub struct TicketWatchInventoryQuery {
    pub since: Option<String>,
    pub fallback_match_id: Option<i64>,
}

pub async fn get_current_ticket_watch_match_handler(
    State(state): State<Arc<TicketWatchWebState>>,
) -> Result<Json<TicketWatchCurrentMatchDto>, (StatusCode, String)> {
    let view = state
        .get_current_ticket_watch_match_use_case
        .execute()
        .await
        .map_err(map_ticket_watch_error)?;

    Ok(Json(view.into()))
}

pub async fn get_current_ticket_watch_board_handler(
    State(state): State<Arc<TicketWatchWebState>>,
    headers: HeaderMap,
) -> Result<Json<TicketWatchCurrentBoardDto>, (StatusCode, String)> {
    let viewer_user_id = authenticate_user_optional(&headers, state.token_port.as_ref());
    let view = state
        .get_current_ticket_watch_board_use_case
        .execute(viewer_user_id)
        .await
        .map_err(map_ticket_watch_error)?;

    Ok(Json(view.into()))
}

pub async fn list_ticket_watch_matches_handler(
    State(state): State<Arc<TicketWatchWebState>>,
) -> Result<Json<Vec<TicketWatchMatchSummaryDto>>, (StatusCode, String)> {
    let matches = state
        .list_ticket_watch_matches_use_case
        .execute()
        .await
        .map_err(map_ticket_watch_error)?;

    Ok(Json(matches.into_iter().map(Into::into).collect()))
}

pub async fn list_ticket_watch_regions_handler(
    State(state): State<Arc<TicketWatchWebState>>,
) -> Result<Json<Vec<TicketWatchRegionDto>>, (StatusCode, String)> {
    let regions = state
        .list_ticket_watch_regions_use_case
        .execute()
        .await
        .map_err(map_ticket_watch_error)?;

    Ok(Json(regions.into_iter().map(Into::into).collect()))
}

pub async fn get_match_ticket_inventory_handler(
    State(state): State<Arc<TicketWatchWebState>>,
    Path(match_id): Path<i64>,
    Query(query): Query<TicketWatchInventoryQuery>,
) -> Result<Json<Vec<TicketWatchInventoryEntryDto>>, (StatusCode, String)> {
    let inventory = state
        .get_match_ticket_inventory_use_case
        .execute_with_since(match_id, query.fallback_match_id, query.since.as_deref())
        .await
        .map_err(map_ticket_watch_error)?;

    Ok(Json(inventory.into_iter().map(Into::into).collect()))
}

pub async fn get_match_block_interests_handler(
    State(state): State<Arc<TicketWatchWebState>>,
    headers: HeaderMap,
    Path(match_id): Path<i64>,
) -> Result<Json<Vec<TicketWatchBlockInterestDto>>, (StatusCode, String)> {
    let viewer_user_id = authenticate_user_optional(&headers, state.token_port.as_ref());
    let interests = state
        .get_match_block_interests_use_case
        .execute(match_id, viewer_user_id)
        .await
        .map_err(map_ticket_watch_error)?;

    Ok(Json(interests.into_iter().map(Into::into).collect()))
}

pub async fn get_match_tracked_interests_handler(
    State(state): State<Arc<TicketWatchWebState>>,
    headers: HeaderMap,
    Path(match_id): Path<i64>,
) -> Result<Json<Vec<TicketWatchTrackedInterestDto>>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let tracked_interests = state
        .get_match_tracked_interests_use_case
        .execute(match_id, user_id)
        .await
        .map_err(map_ticket_watch_error)?;

    Ok(Json(
        tracked_interests.into_iter().map(Into::into).collect(),
    ))
}

pub async fn toggle_match_block_interest_handler(
    State(state): State<Arc<TicketWatchWebState>>,
    headers: HeaderMap,
    Path(match_id): Path<i64>,
    Json(request): Json<ToggleTicketWatchBlockInterestRequest>,
) -> Result<Json<TicketWatchBlockInterestDto>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let interest = state
        .toggle_match_block_interest_use_case
        .execute(match_id, user_id, &request.block_name)
        .await
        .map_err(map_ticket_watch_error)?;

    Ok(Json(interest.into()))
}

fn authenticate_user(
    headers: &HeaderMap,
    token_port: &dyn TokenPort,
) -> Result<Uuid, (StatusCode, String)> {
    let token =
        extract_bearer_token(headers).ok_or((StatusCode::UNAUTHORIZED, "请先登录".to_string()))?;

    token_port
        .verify_token(token)
        .map(|claims| claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "请先登录".to_string()))
}

fn authenticate_user_optional(headers: &HeaderMap, token_port: &dyn TokenPort) -> Option<Uuid> {
    let token = extract_bearer_token(headers)?;
    token_port.verify_token(token).ok().map(|claims| claims.sub)
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    header_value.strip_prefix("Bearer ")
}

fn map_ticket_watch_error(error: anyhow::Error) -> (StatusCode, String) {
    (StatusCode::BAD_GATEWAY, error.to_string())
}
