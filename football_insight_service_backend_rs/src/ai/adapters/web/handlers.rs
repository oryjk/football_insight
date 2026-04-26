use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
};
use futures_util::StreamExt;
use serde_json::json;

use crate::{
    ai::{
        adapters::web::dto::{AiChatRequest, AiChatResponseDto},
        application::chat_with_model::{ChatWithModelInput, ChatWithModelUseCase},
        domain::chat::{AiChatActor, AiChatMessage, AiChatStreamEvent},
    },
    auth::application::get_current_user::GetCurrentUserUseCase,
};

#[derive(Clone)]
pub struct AiWebState {
    pub chat_with_model_use_case: Arc<ChatWithModelUseCase>,
    pub get_current_user_use_case: Arc<GetCurrentUserUseCase>,
}

pub async fn ai_chat_handler(
    State(state): State<Arc<AiWebState>>,
    headers: HeaderMap,
    Json(request): Json<AiChatRequest>,
) -> Result<Json<AiChatResponseDto>, (StatusCode, String)> {
    let input = resolve_ai_chat_input(&state, headers, request).await?;

    let result = state
        .chat_with_model_use_case
        .execute(input)
        .await
        .map_err(map_ai_error)?;

    Ok(Json(result.into()))
}

pub async fn ai_chat_stream_handler(
    State(state): State<Arc<AiWebState>>,
    headers: HeaderMap,
    Json(request): Json<AiChatRequest>,
) -> Result<Response, (StatusCode, String)> {
    let input = resolve_ai_chat_input(&state, headers, request).await?;

    let stream = match state.chat_with_model_use_case.stream(input).await {
        Ok(stream) => stream,
        Err(error) => {
            let (status, message) = map_ai_error(error);
            let event_stream = async_stream::stream! {
                yield Ok::<Event, Infallible>(Event::default().event("error").data(json!({
                    "status": status.as_u16(),
                    "message": message,
                }).to_string()));
            };

            return Ok(Sse::new(event_stream)
                .keep_alive(
                    KeepAlive::new()
                        .interval(Duration::from_secs(15))
                        .text("keep-alive"),
                )
                .into_response());
        }
    };

    let event_stream = async_stream::stream! {
        let mut stream = stream;

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(AiChatStreamEvent::Started { model }) => {
                    yield Ok::<Event, Infallible>(Event::default().event("started").data(json!({
                        "model": model,
                    }).to_string()));
                }
                Ok(AiChatStreamEvent::Delta { content }) => {
                    if content.is_empty() {
                        continue;
                    }

                    yield Ok::<Event, Infallible>(Event::default().event("delta").data(json!({
                        "content": content,
                    }).to_string()));
                }
                Ok(AiChatStreamEvent::Completed { model, reply }) => {
                    yield Ok::<Event, Infallible>(Event::default().event("done").data(json!({
                        "model": model,
                        "reply": reply,
                    }).to_string()));
                    break;
                }
                Err(error) => {
                    yield Ok::<Event, Infallible>(Event::default().event("error").data(json!({
                        "message": map_ai_error(error).1,
                    }).to_string()));
                    break;
                }
            }
        }
    };

    Ok(Sse::new(event_stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive"),
        )
        .into_response())
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let header_value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    header_value.strip_prefix("Bearer ")
}

async fn resolve_ai_chat_input(
    state: &Arc<AiWebState>,
    headers: HeaderMap,
    request: AiChatRequest,
) -> Result<ChatWithModelInput, (StatusCode, String)> {
    let token =
        extract_bearer_token(&headers).ok_or((StatusCode::UNAUTHORIZED, "请先登录".to_string()))?;

    let user = state
        .get_current_user_use_case
        .execute(token)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "请先登录".to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "请先登录".to_string()))?;

    let history = request
        .history
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<AiChatMessage>, _>>()
        .map_err(map_ai_error)?;

    Ok(ChatWithModelInput {
        actor: AiChatActor {
            display_name: user
                .display_name
                .unwrap_or_else(|| user.account_identifier.clone()),
            membership_tier: user.membership_tier,
        },
        message: request.message,
        history,
    })
}

fn map_ai_error(error: anyhow::Error) -> (StatusCode, String) {
    let message = error.to_string();

    if message.contains("message is required") {
        return (StatusCode::BAD_REQUEST, "请输入对话内容".to_string());
    }

    if message.contains("message is too long") {
        return (StatusCode::BAD_REQUEST, "单次提问内容过长".to_string());
    }

    if message.contains("history role is invalid") {
        return (StatusCode::BAD_REQUEST, "历史消息角色不合法".to_string());
    }

    if message.contains("ai chat is not configured") {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "AI 服务暂未配置".to_string(),
        );
    }

    if message.contains("1113") || message.contains("余额不足") || message.contains("无可用资源包")
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "AI 服务当前余额不足，请联系管理员处理".to_string(),
        );
    }

    if message.contains("1305") || message.contains("访问量过大") {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "AI 服务当前访问量过大，请稍后再试".to_string(),
        );
    }

    if message.contains("timed out") || message.contains("deadline has elapsed") {
        return (
            StatusCode::GATEWAY_TIMEOUT,
            "AI 服务响应超时，请稍后重试".to_string(),
        );
    }

    if message.contains("api key")
        || message.contains("OpenAI")
        || message.contains("openai")
        || message.contains("status code")
        || message.contains("provider")
        || message.contains("completion")
    {
        return (
            StatusCode::BAD_GATEWAY,
            format!("AI 服务调用失败: {message}"),
        );
    }

    (StatusCode::INTERNAL_SERVER_ERROR, message)
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::map_ai_error;

    #[test]
    fn map_ai_error_translates_provider_rate_limit() {
        let (status, message) = map_ai_error(anyhow::anyhow!(
            "{}",
            "provider status code 429: {\"error\":{\"code\":\"1305\",\"message\":\"该模型当前访问量过大，请您稍后再试\"}}"
        ));

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(message, "AI 服务当前访问量过大，请稍后再试");
    }
}
