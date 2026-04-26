use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    auth::ports::token_port::TokenPort,
    team_board::{
        adapters::web::dto::{
            AddTeamBoardCommentRequest, CreateTeamBoardPostRequest, TeamBoardCommentDto,
            TeamBoardLikeSummaryDto, TeamBoardPostDto, TeamBoardViewDto,
        },
        application::{
            add_team_board_comment::{AddTeamBoardCommentInput, AddTeamBoardCommentUseCase},
            create_team_board_post::{CreateTeamBoardPostInput, CreateTeamBoardPostUseCase},
            get_team_board::GetTeamBoardUseCase,
            toggle_team_board_post_like::ToggleTeamBoardPostLikeUseCase,
        },
    },
};

#[derive(Clone)]
pub struct TeamBoardWebState {
    pub get_team_board_use_case: Arc<GetTeamBoardUseCase>,
    pub create_team_board_post_use_case: Arc<CreateTeamBoardPostUseCase>,
    pub add_team_board_comment_use_case: Arc<AddTeamBoardCommentUseCase>,
    pub toggle_team_board_post_like_use_case: Arc<ToggleTeamBoardPostLikeUseCase>,
    pub token_port: Arc<dyn TokenPort>,
}

pub async fn get_team_board_handler(
    State(state): State<Arc<TeamBoardWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
) -> Result<Json<TeamBoardViewDto>, (StatusCode, String)> {
    let viewer_user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let board = state
        .get_team_board_use_case
        .execute(team_id, viewer_user_id)
        .await
        .map_err(map_team_board_error)?;

    Ok(Json(board.into()))
}

pub async fn create_team_board_post_handler(
    State(state): State<Arc<TeamBoardWebState>>,
    headers: HeaderMap,
    Path(team_id): Path<i64>,
    Json(request): Json<CreateTeamBoardPostRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let author_user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let post = state
        .create_team_board_post_use_case
        .execute(CreateTeamBoardPostInput {
            team_id,
            author_user_id,
            insight_kind: request.insight_kind,
            title: request.title,
            commentary: request.commentary,
        })
        .await
        .map_err(map_team_board_error)?;

    Ok((StatusCode::CREATED, Json(TeamBoardPostDto::from(post))))
}

pub async fn add_team_board_comment_handler(
    State(state): State<Arc<TeamBoardWebState>>,
    headers: HeaderMap,
    Path(post_id): Path<Uuid>,
    Json(request): Json<AddTeamBoardCommentRequest>,
) -> Result<(StatusCode, Json<TeamBoardCommentDto>), (StatusCode, String)> {
    let author_user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let comment = state
        .add_team_board_comment_use_case
        .execute(AddTeamBoardCommentInput {
            post_id,
            author_user_id,
            content: request.content,
        })
        .await
        .map_err(map_team_board_error)?;

    Ok((StatusCode::CREATED, Json(comment.into())))
}

pub async fn toggle_team_board_post_like_handler(
    State(state): State<Arc<TeamBoardWebState>>,
    headers: HeaderMap,
    Path(post_id): Path<Uuid>,
) -> Result<Json<TeamBoardLikeSummaryDto>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let summary = state
        .toggle_team_board_post_like_use_case
        .execute(post_id, user_id)
        .await
        .map_err(map_team_board_error)?;

    Ok(Json(summary.into()))
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

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    header_value.strip_prefix("Bearer ")
}

fn map_team_board_error(error: anyhow::Error) -> (StatusCode, String) {
    let message = error.to_string();

    if message.contains("title is required") {
        return (StatusCode::BAD_REQUEST, "请输入标题".to_string());
    }

    if message.contains("commentary is required") {
        return (StatusCode::BAD_REQUEST, "请输入一句点评".to_string());
    }

    if message.contains("comment content is required") {
        return (StatusCode::BAD_REQUEST, "请输入评论内容".to_string());
    }

    if message.contains("team insight not found") {
        return (StatusCode::NOT_FOUND, "未找到对应球队洞察".to_string());
    }

    if message.contains("not found") {
        return (StatusCode::NOT_FOUND, "内容不存在".to_string());
    }

    (StatusCode::INTERNAL_SERVER_ERROR, message)
}
