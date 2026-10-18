use crate::{
    auth::ports::token_port::TokenPort,
    share_code::{
        application::ShareCodeService,
        domain::{CodeImage, ShareCodeError},
    },
};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct WebState {
    pub service: Arc<ShareCodeService>,
    pub token: Arc<dyn TokenPort>,
}

#[derive(Serialize)]
pub struct CodeResponse {
    image_base64: String,
    mime_type: String,
}

impl From<CodeImage> for CodeResponse {
    fn from(image: CodeImage) -> Self {
        Self {
            image_base64: image.image_base64,
            mime_type: image.mime_type,
        }
    }
}

pub fn routes(state: WebState) -> Router {
    Router::new()
        .route("/api/v1/share-codes/ticket-watch", post(ticket_watch))
        .with_state(state)
}

async fn ticket_watch(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Result<Json<CodeResponse>, (StatusCode, String)> {
    headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .and_then(|value| state.token.verify_token(value).ok())
        .ok_or((StatusCode::UNAUTHORIZED, "请先登录".into()))?;
    state
        .service
        .ticket_watch()
        .await
        .map(|image| Json(image.into()))
        .map_err(|error| match error {
            ShareCodeError::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, error.to_string()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::adapters::security::jwt_token_port::JwtTokenPort,
        share_code::adapters::wechat_code::WechatTicketWatchCode,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{Duration, Utc};
    use tower::ServiceExt;
    use uuid::Uuid;

    fn fixture() -> (Router, Arc<JwtTokenPort>) {
        let token = Arc::new(JwtTokenPort::new("share-code-test-secret".into()));
        (
            routes(WebState {
                service: Arc::new(ShareCodeService::new(Arc::new(WechatTicketWatchCode::new(
                    String::new(),
                    String::new(),
                )))),
                token: token.clone(),
            }),
            token,
        )
    }

    #[tokio::test]
    async fn share_code_rejects_missing_malformed_and_expired_access_tokens() {
        let (app, token) = fixture();
        let expired = token
            .issue_token(Uuid::nil(), "test", Utc::now() - Duration::hours(1))
            .unwrap();
        for authorization in [
            None,
            Some("invalid".into()),
            Some("Bearer invalid".into()),
            Some(format!("Bearer {expired}")),
        ] {
            let mut request = Request::post("/api/v1/share-codes/ticket-watch");
            if let Some(value) = authorization {
                request = request.header("Authorization", value);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
    }

    #[tokio::test]
    async fn logged_in_user_without_wechat_credentials_receives_service_unavailable() {
        let (app, token) = fixture();
        let access = token
            .issue_token(Uuid::nil(), "test", Utc::now() + Duration::hours(1))
            .unwrap();
        let response = app
            .oneshot(
                Request::post("/api/v1/share-codes/ticket-watch")
                    .header("Authorization", format!("Bearer {access}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
