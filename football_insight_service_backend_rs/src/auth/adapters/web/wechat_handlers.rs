use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::auth::application::handle_wechat_webhook::{
    HandleWechatWebhookUseCase, WechatMessageInput, WechatVerificationInput,
};

#[derive(Debug, Deserialize)]
pub struct WechatWebhookQuery {
    pub signature: Option<String>,
    pub msg_signature: Option<String>,
    pub timestamp: String,
    pub nonce: String,
    pub echostr: Option<String>,
}

pub async fn wechat_verify_handler(
    Query(query): Query<WechatWebhookQuery>,
    State(use_case): State<Arc<HandleWechatWebhookUseCase>>,
) -> Result<Response, (StatusCode, String)> {
    let signature = pick_signature(&query)?;
    let echostr = query
        .echostr
        .ok_or((StatusCode::BAD_REQUEST, "echostr is required".to_string()))?;

    let plain = use_case
        .verify_endpoint(WechatVerificationInput {
            signature,
            timestamp: query.timestamp,
            nonce: query.nonce,
            echostr,
        })
        .map_err(|error| {
            tracing::warn!(error = %error, "wechat verify handler failed");
            map_wechat_error(error)
        })?;

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "text/plain; charset=utf-8".parse().unwrap());
    Ok((headers, plain).into_response())
}

pub async fn wechat_message_handler(
    Query(query): Query<WechatWebhookQuery>,
    State(use_case): State<Arc<HandleWechatWebhookUseCase>>,
    body: String,
) -> Result<Response, (StatusCode, String)> {
    let signature = pick_message_signature(&query, &body)?;
    let response_xml = use_case
        .handle_message(WechatMessageInput {
            signature,
            timestamp: query.timestamp,
            nonce: query.nonce,
            body,
        })
        .await
        .map_err(|error| {
            tracing::warn!(error = %error, "wechat message handler failed");
            map_wechat_error(error)
        })?;

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        "application/xml; charset=utf-8".parse().unwrap(),
    );
    Ok((
        headers,
        response_xml.unwrap_or_else(|| "success".to_string()),
    )
        .into_response())
}

fn pick_signature(query: &WechatWebhookQuery) -> Result<String, (StatusCode, String)> {
    query
        .msg_signature
        .clone()
        .or_else(|| query.signature.clone())
        .ok_or((StatusCode::BAD_REQUEST, "signature is required".to_string()))
}

fn pick_message_signature(
    query: &WechatWebhookQuery,
    body: &str,
) -> Result<String, (StatusCode, String)> {
    if body.contains("<Encrypt>") || body.contains("<Encrypt><![CDATA[") {
        query
            .msg_signature
            .clone()
            .or_else(|| query.signature.clone())
            .ok_or((
                StatusCode::BAD_REQUEST,
                "msg_signature is required".to_string(),
            ))
    } else {
        query
            .signature
            .clone()
            .or_else(|| query.msg_signature.clone())
            .ok_or((StatusCode::BAD_REQUEST, "signature is required".to_string()))
    }
}

fn map_wechat_error(error: anyhow::Error) -> (StatusCode, String) {
    let message = error.to_string();

    if message.contains("signature")
        || message.contains("echostr")
        || message.contains("Encrypt")
        || message.contains("wechat")
    {
        return (StatusCode::BAD_REQUEST, message);
    }

    (StatusCode::INTERNAL_SERVER_ERROR, message)
}
