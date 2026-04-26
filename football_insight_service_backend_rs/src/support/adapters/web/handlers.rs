use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use uuid::Uuid;

use crate::{
    auth::ports::token_port::TokenPort,
    support::{
        adapters::web::dto::{
            CastSupportVoteRequest, SetFavoriteTeamRequest, SupportMatchDetailDto,
            SupportProfileDto, SupportTeamDto,
        },
        application::{
            cast_match_support_vote::CastMatchSupportVoteUseCase,
            get_match_support_detail::GetMatchSupportDetailUseCase,
            get_support_profile::GetSupportProfileUseCase,
            list_support_teams::ListSupportTeamsUseCase, set_favorite_team::SetFavoriteTeamUseCase,
        },
    },
};

#[derive(Clone)]
pub struct SupportWebState {
    pub list_support_teams_use_case: Arc<ListSupportTeamsUseCase>,
    pub get_support_profile_use_case: Arc<GetSupportProfileUseCase>,
    pub set_favorite_team_use_case: Arc<SetFavoriteTeamUseCase>,
    pub get_match_support_detail_use_case: Arc<GetMatchSupportDetailUseCase>,
    pub cast_match_support_vote_use_case: Arc<CastMatchSupportVoteUseCase>,
    pub token_port: Arc<dyn TokenPort>,
}

pub async fn list_support_teams_handler(
    State(state): State<Arc<SupportWebState>>,
) -> Result<Json<Vec<SupportTeamDto>>, (StatusCode, String)> {
    let teams = state
        .list_support_teams_use_case
        .execute()
        .await
        .map_err(map_support_error)?;

    Ok(Json(teams.into_iter().map(Into::into).collect()))
}

pub async fn get_support_profile_handler(
    State(state): State<Arc<SupportWebState>>,
    headers: HeaderMap,
) -> Result<Json<SupportProfileDto>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let profile = state
        .get_support_profile_use_case
        .execute(user_id)
        .await
        .map_err(map_support_error)?;

    Ok(Json(profile.into()))
}

pub async fn set_favorite_team_handler(
    State(state): State<Arc<SupportWebState>>,
    headers: HeaderMap,
    Json(request): Json<SetFavoriteTeamRequest>,
) -> Result<Json<SupportTeamDto>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let team = state
        .set_favorite_team_use_case
        .execute(user_id, request.team_id)
        .await
        .map_err(map_support_error)?;

    Ok(Json(team.into()))
}

pub async fn get_match_support_detail_handler(
    State(state): State<Arc<SupportWebState>>,
    headers: HeaderMap,
    Path(match_id): Path<i64>,
) -> Result<Json<SupportMatchDetailDto>, (StatusCode, String)> {
    let viewer_user_id = authenticate_user_optional(&headers, state.token_port.as_ref());
    let detail = state
        .get_match_support_detail_use_case
        .execute(match_id, viewer_user_id)
        .await
        .map_err(map_support_error)?
        .ok_or((StatusCode::NOT_FOUND, "未找到对应比赛".to_string()))?;

    Ok(Json(detail.into()))
}

pub async fn cast_support_vote_handler(
    State(state): State<Arc<SupportWebState>>,
    headers: HeaderMap,
    Path(match_id): Path<i64>,
    Json(request): Json<CastSupportVoteRequest>,
) -> Result<Json<SupportMatchDetailDto>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let detail = state
        .cast_match_support_vote_use_case
        .execute(user_id, match_id, request.supported_team_id)
        .await
        .map_err(map_support_error)?;

    Ok(Json(detail.into()))
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

fn map_support_error(error: anyhow::Error) -> (StatusCode, String) {
    let message = error.to_string();

    if message.contains("favorite team is required") {
        return (StatusCode::BAD_REQUEST, "请先关注你的主队".to_string());
    }

    if message.contains("favorite team not found") {
        return (StatusCode::NOT_FOUND, "未找到该球队".to_string());
    }

    if message.contains("supported team must match favorite team") {
        return (
            StatusCode::BAD_REQUEST,
            "只能为你关注的主队助力".to_string(),
        );
    }

    if message.contains("supported team does not belong to this match") {
        return (
            StatusCode::BAD_REQUEST,
            "所选球队不在本场对阵中".to_string(),
        );
    }

    if message.contains("support window is not open") {
        return (StatusCode::BAD_REQUEST, "当前不在助力开放时间".to_string());
    }

    if message.contains("already supported") {
        return (StatusCode::CONFLICT, "本场比赛你已经助力过了".to_string());
    }

    if message.contains("support match not found") {
        return (StatusCode::NOT_FOUND, "未找到对应比赛".to_string());
    }

    if message.contains("user not found") {
        return (StatusCode::NOT_FOUND, "未找到当前用户".to_string());
    }

    (StatusCode::INTERNAL_SERVER_ERROR, message)
}
