use std::sync::Arc;

use axum::{Json, extract::Query, http::HeaderMap, http::StatusCode};

use crate::mini_review::{
    adapters::web::dto::{
        AllocateRequest, ReviewStatusDto, ReviewStatusQuery, SetReviewStatusRequest,
    },
    application::{
        allocate_review_version::{AllocateCommand, AllocateReviewVersionUseCase},
        get_review_status::GetReviewStatusUseCase,
        set_review_status_by_project_version::{
            SetReviewStatusByProjectVersionUseCase, SetReviewStatusCommand,
        },
    },
};

fn map_use_case_error(error: anyhow::Error) -> (StatusCode, String) {
    let message = error.to_string();
    if message.contains("required") || message.contains("invalid") {
        (StatusCode::BAD_REQUEST, message)
    } else if message.contains("not registered") {
        (StatusCode::NOT_FOUND, message)
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

/// 常数时间比较，避免密钥逐字节泄露；长度不同直接失败（与 Go 版 crypto/subtle 行为一致）。
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

/// 静态 API key 鉴权（构建脚本场景无用户会话）；未配置密钥时登记接口不开放。
pub fn authorize_api_key(headers: &HeaderMap, configured_key: &Option<String>) -> bool {
    let Some(configured_key) = configured_key.as_deref().map(str::trim) else {
        return false;
    };
    if configured_key.is_empty() {
        return false;
    }
    let Some(presented) = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
    else {
        return false;
    };
    constant_time_eq(presented.as_bytes(), configured_key.as_bytes())
}

pub fn api_key_unauthorized() -> (StatusCode, String) {
    (
        StatusCode::FORBIDDEN,
        "登记接口未开放或密钥无效".to_string(),
    )
}

pub async fn get_review_status_handler(
    use_case: Arc<GetReviewStatusUseCase>,
    Query(query): Query<ReviewStatusQuery>,
) -> Result<Json<ReviewStatusDto>, (StatusCode, String)> {
    let status = use_case
        .execute(&query.project_code, &query.version)
        .await
        .map_err(map_use_case_error)?;

    Ok(Json(status.into()))
}

pub async fn allocate_handler(
    use_case: Arc<AllocateReviewVersionUseCase>,
    configured_key: Option<String>,
    headers: HeaderMap,
    Json(request): Json<AllocateRequest>,
) -> Result<Json<ReviewStatusDto>, (StatusCode, String)> {
    if !authorize_api_key(&headers, &configured_key) {
        return Err(api_key_unauthorized());
    }

    let status = use_case
        .execute(AllocateCommand {
            project_code: request.project_code,
            current_version: request.current_version,
            explicit_version: request.version,
        })
        .await
        .map_err(map_use_case_error)?;

    Ok(Json(status.into()))
}

/// 审核状态切换的调用方：构建脚本（静态密钥）或小程序端白名单用户（JWT）。
enum ReviewControlActor {
    Script,
    MiniProgramUser,
}

/// PUT review-status 双通道鉴权：
/// 1. 有效的 X-Api-Key（构建/运维脚本，与 allocate 一致）；
/// 2. 有效的 Bearer JWT 且用户在 MINI_REVIEW_CONTROL_USER_IDS 白名单内（小程序「设置」入口）。
fn authorize_review_control(
    headers: &HeaderMap,
    configured_key: &Option<String>,
    token_port: &dyn crate::auth::ports::token_port::TokenPort,
    control_user_ids: &[uuid::Uuid],
) -> Result<ReviewControlActor, (StatusCode, String)> {
    if authorize_api_key(headers, configured_key) {
        return Ok(ReviewControlActor::Script);
    }

    if let Some(token) = extract_bearer_token(headers) {
        return match token_port.verify_token(token) {
            Ok(claims) if control_user_ids.contains(&claims.sub) => Ok(ReviewControlActor::MiniProgramUser),
            Ok(_) => Err((StatusCode::FORBIDDEN, "当前账号无权切换审核状态".to_string())),
            Err(_) => Err((StatusCode::UNAUTHORIZED, "请先登录".to_string())),
        };
    }

    Err(api_key_unauthorized())
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(axum::http::header::AUTHORIZATION)?.to_str().ok()?;
    header_value.strip_prefix("Bearer ")
}

/// 小程序端切换时的状态文案，与注册系统口径一致。
fn mini_program_status_text(is_reviewing: bool) -> String {
    if is_reviewing {
        "审核中（小程序端切换）".to_string()
    } else {
        "已过审（小程序端切换）".to_string()
    }
}

pub async fn set_review_status_handler(
    use_case: Arc<SetReviewStatusByProjectVersionUseCase>,
    configured_key: Option<String>,
    token_port: std::sync::Arc<dyn crate::auth::ports::token_port::TokenPort>,
    control_user_ids: Vec<uuid::Uuid>,
    headers: HeaderMap,
    Json(request): Json<SetReviewStatusRequest>,
) -> Result<Json<ReviewStatusDto>, (StatusCode, String)> {
    let actor = authorize_review_control(&headers, &configured_key, token_port.as_ref(), &control_user_ids)?;

    let status_text = match actor {
        ReviewControlActor::Script => request.status_text,
        ReviewControlActor::MiniProgramUser => Some(mini_program_status_text(request.is_reviewing)),
    };

    let status = use_case
        .execute(SetReviewStatusCommand {
            project_code: request.project_code,
            version: request.version,
            is_reviewing: request.is_reviewing,
            status_text,
        })
        .await
        .map_err(map_use_case_error)?;

    Ok(Json(status.into()))
}

#[cfg(test)]
mod tests {
    use super::{api_key_unauthorized, authorize_api_key, constant_time_eq};
    use axum::http::HeaderMap;

    #[test]
    fn constant_time_eq_compares_content_not_prefix() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(!constant_time_eq(b"", b"abc"));
    }

    #[test]
    fn authorize_api_key_rejects_missing_or_mismatched_keys() {
        let mut headers = HeaderMap::new();
        assert!(!authorize_api_key(&headers, &Some("secret".to_string())));

        headers.insert("x-api-key", "wrong".parse().unwrap());
        assert!(!authorize_api_key(&headers, &Some("secret".to_string())));
        assert!(!authorize_api_key(&headers, &None));
        assert!(!authorize_api_key(&headers, &Some("  ".to_string())));

        headers.insert("x-api-key", "secret".parse().unwrap());
        assert!(authorize_api_key(&headers, &Some("secret".to_string())));
        assert!(authorize_api_key(&headers, &Some(" secret ".to_string())));
    }

    #[test]
    fn api_key_unauthorized_returns_forbidden() {
        let (status, message) = api_key_unauthorized();
        assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
        assert!(message.contains("密钥无效"));
    }
}
