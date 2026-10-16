use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
    response::IntoResponse,
};

use crate::{
    auth::ports::{token_port::TokenPort, user_membership_port::UserMembershipPort},
    payment::{
        adapters::web::dto::{
            CreateMembershipOrderRequest, CreateOrderResponse, MembershipProductResponse,
            OrderStatusResponse,
        },
        application::{
            create_membership_order::{CreateMembershipOrderInput, CreateMembershipOrderUseCase},
            get_membership_product::GetMembershipProductUseCase,
            get_order_status::{GetOrderStatusInput, GetOrderStatusUseCase},
            handle_wechat_notify::{HandleWechatNotifyUseCase, NotifyHandleResult},
        },
    },
};

#[derive(Clone)]
pub struct PaymentWebState {
    pub virtual_payment: Arc<crate::payment::application::virtual_payment::VirtualPaymentService>,
    pub virtual_webhook:
        Option<Arc<dyn crate::payment::ports::virtual_payment_port::VirtualWebhookPort>>,
    pub create_membership_order_use_case: Arc<CreateMembershipOrderUseCase>,
    pub get_membership_product_use_case: Arc<GetMembershipProductUseCase>,
    pub get_order_status_use_case: Arc<GetOrderStatusUseCase>,
    pub handle_wechat_notify_use_case: Arc<HandleWechatNotifyUseCase>,
    pub token_port: Arc<dyn TokenPort>,
    pub user_membership_port: Arc<dyn UserMembershipPort>,
    pub wechat_pay_api_key: String,
}

pub(super) fn authenticate_user(
    headers: &HeaderMap,
    token_port: &dyn TokenPort,
) -> Result<uuid::Uuid, (StatusCode, String)> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "请先登录".to_string()))?;

    token_port
        .verify_token(token)
        .map(|claims| claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "请先登录".to_string()))
}

pub(crate) fn client_capabilities(headers: &HeaderMap) -> Vec<String> {
    headers
        .get("X-Client-Capabilities")
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.split([',', ' '])
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn try_authenticate_user(headers: &HeaderMap, token_port: &dyn TokenPort) -> Option<uuid::Uuid> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))?;

    token_port.verify_token(token).map(|claims| claims.sub).ok()
}

pub async fn get_membership_product_handler(
    State(state): State<Arc<PaymentWebState>>,
    headers: HeaderMap,
) -> Result<Json<MembershipProductResponse>, (StatusCode, String)> {
    let user_id = try_authenticate_user(&headers, state.token_port.as_ref());
    let current_tier = match user_id {
        Some(user_id) => state
            .user_membership_port
            .get_user_membership_tier(user_id)
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "failed to get user membership tier");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "获取会员信息失败".to_string(),
                )
            })?,
        None => None,
    };

    let product = state
        .get_membership_product_use_case
        .execute_catalog_for_user(
            user_id,
            current_tier.as_deref(),
            &client_capabilities(&headers),
        )
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "failed to get membership product");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "获取会员信息失败".to_string(),
            )
        })?;
    let product = state
        .get_membership_product_use_case
        .with_duration_for_tier(product, current_tier.as_deref(), user_id)
        .await
        .map_err(|error| {
            tracing::error!(%error,"failed to get membership duration");
            (StatusCode::INTERNAL_SERVER_ERROR, "获取会员信息失败".into())
        })?;
    Ok(Json(MembershipProductResponse::from_product_for_tier(
        product,
        current_tier.as_deref(),
    )))
}

pub async fn create_membership_order_handler(
    State(state): State<Arc<PaymentWebState>>,
    headers: HeaderMap,
    payload: Option<Json<CreateMembershipOrderRequest>>,
) -> Result<Json<CreateOrderResponse>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    let target_tier = payload
        .and_then(|Json(payload)| payload.target_tier)
        .unwrap_or_else(|| "V9".to_string())
        .trim()
        .to_uppercase();

    let capabilities = client_capabilities(&headers);
    let current_tier = state
        .user_membership_port
        .get_user_membership_tier(user_id)
        .await
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("创建订单失败: {}", error),
            )
        })?;
    let catalog = state
        .get_membership_product_use_case
        .execute_purchase_catalog(
            user_id,
            current_tier.as_deref(),
            &capabilities,
            &target_tier,
        )
        .await
        .map_err(|error| {
            if error
                .downcast_ref::<crate::payment::domain::product_kind::ProductSpecError>()
                .is_some()
            {
                return (StatusCode::BAD_REQUEST, error.to_string());
            }
            tracing::error!(error = %error, "failed to get membership product for order");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "获取会员信息失败".to_string(),
            )
        })?;
    let product = catalog
        .find_option(&target_tier)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "请选择有效的会员档位".to_string()))?;

    let result = state
        .create_membership_order_use_case
        .execute_with_capabilities(
            CreateMembershipOrderInput {
                user_id,
                target_tier: product.target_tier.clone(),
                products: catalog.products,
            },
            Some(crate::payment::adapters::web::event_handlers::attempt_from_headers(&headers)),
            &capabilities,
        )
        .await
        .map_err(|error| {
            if error
                .downcast_ref::<crate::payment::domain::product_kind::ProductSpecError>()
                .is_some()
            {
                return (StatusCode::BAD_REQUEST, error.to_string());
            }
            if let Some(error) =
                error.downcast_ref::<crate::payment::domain::virtual_payment::VirtualPaymentError>()
            {
                return virtual_error(error);
            }
            let msg = format!("{}", error);
            if msg.contains("高于当前等级") || msg.contains("请先绑定微信") {
                return (StatusCode::BAD_REQUEST, msg);
            }
            tracing::error!(error = %error, "failed to create membership order");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("创建订单失败: {}", error),
            )
        })?;

    Ok(Json(CreateOrderResponse {
        order_no: result.order_no,
        params: result.wx_pay_params.into(),
    }))
}

pub async fn get_order_status_handler(
    State(state): State<Arc<PaymentWebState>>,
    headers: HeaderMap,
    Path(order_no): Path<String>,
) -> Result<Json<OrderStatusResponse>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;

    if state.virtual_payment.available()
        && let Err(error) = state
            .virtual_payment
            .reconcile(&order_no, Some(user_id))
            .await
    {
        if matches!(
            error.downcast_ref::<crate::payment::domain::virtual_payment::VirtualPaymentError>(),
            Some(crate::payment::domain::virtual_payment::VirtualPaymentError::IdentityMismatch)
        ) {
            return Err((StatusCode::FORBIDDEN, "无权查看此订单".into()));
        }
        tracing::warn!(order_no=%order_no,"virtual payment reconciliation deferred");
    }

    let order = state
        .get_order_status_use_case
        .execute(GetOrderStatusInput { order_no, user_id })
        .await
        .map_err(|error| {
            let msg = format!("{}", error);
            if msg.contains("无权") {
                return (StatusCode::FORBIDDEN, msg);
            }
            tracing::error!(error = %error, "failed to get order status");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("查询订单失败: {}", error),
            )
        })?;

    let Some(order) = order else {
        return Err((StatusCode::NOT_FOUND, "订单不存在".to_string()));
    };

    Ok(Json(OrderStatusResponse {
        order_no: order.order_no,
        status: order.status.as_str().to_string(),
        amount_cents: order.amount_cents,
        paid_at: order.paid_at.map(|dt| dt.to_rfc3339()),
    }))
}

pub(crate) fn virtual_error(
    error: &crate::payment::domain::virtual_payment::VirtualPaymentError,
) -> (StatusCode, String) {
    use crate::payment::domain::virtual_payment::VirtualPaymentError::*;
    let status = match error {
        Unavailable | ProductUnavailable | CatalogUnavailable | Provider => {
            StatusCode::SERVICE_UNAVAILABLE
        }
        IdentityMismatch => StatusCode::FORBIDDEN,
        SessionExpired | InvalidOrder => StatusCode::BAD_REQUEST,
    };
    (status, error.to_string())
}

#[derive(serde::Deserialize)]
pub struct PrepareVirtualSession {
    code: String,
}
pub async fn prepare_virtual_session(
    State(state): State<Arc<PaymentWebState>>,
    headers: HeaderMap,
    Json(input): Json<PrepareVirtualSession>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = authenticate_user(&headers, state.token_port.as_ref())?;
    state
        .virtual_payment
        .prepare_session(user_id, &input.code)
        .await
        .map_err(|e| {
            e.downcast_ref::<crate::payment::domain::virtual_payment::VirtualPaymentError>()
                .map(virtual_error)
                .unwrap_or((StatusCode::INTERNAL_SERVER_ERROR, "支付会话准备失败".into()))
        })?;
    Ok(Json(serde_json::json!({"ready":true})))
}
pub async fn virtual_payment_status(
    State(state): State<Arc<PaymentWebState>>,
) -> Json<serde_json::Value> {
    let available = state
        .virtual_payment
        .purchase_available()
        .await
        .unwrap_or(false);
    Json(
        serde_json::json!({"available":available,"message":if available{""}else{"购买服务暂未开放，请稍后再试"}}),
    )
}

#[derive(serde::Deserialize)]
pub struct VirtualWebhookQuery {
    #[serde(default)]
    signature: String,
    #[serde(default)]
    msg_signature: String,
    timestamp: String,
    nonce: String,
    #[serde(default)]
    echostr: String,
}
pub async fn virtual_webhook_challenge(
    State(state): State<Arc<PaymentWebState>>,
    Query(query): Query<VirtualWebhookQuery>,
) -> Result<String, StatusCode> {
    let codec = state
        .virtual_webhook
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let signature = if query.msg_signature.is_empty() {
        &query.signature
    } else {
        &query.msg_signature
    };
    codec
        .challenge(signature, &query.timestamp, &query.nonce, &query.echostr)
        .map_err(|_| StatusCode::FORBIDDEN)
}
pub async fn virtual_webhook_notify(
    State(state): State<Arc<PaymentWebState>>,
    Query(query): Query<VirtualWebhookQuery>,
    body: String,
) -> impl IntoResponse {
    use crate::payment::domain::virtual_payment::VirtualEvent;
    let xml = body.trim_start().starts_with('<');
    let result = async {
        let codec = state
            .virtual_webhook
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("virtual payment disabled"))?;
        let event = codec.decode(&query.msg_signature, &query.timestamp, &query.nonce, &body)?;
        match event {
            VirtualEvent::Delivery(delivery) => {
                state.virtual_payment.deliver_callback(delivery).await?
            }
            VirtualEvent::Refund(refund) => state.virtual_payment.refund(&refund).await?,
            VirtualEvent::RefundQuery => {
                return Err::<(), anyhow::Error>(anyhow::anyhow!(
                    "refund decision requires evidence; leave decision to Apple"
                ));
            }
            VirtualEvent::Ignored => {}
        }
        Ok(())
    }
    .await;
    let code = match result {
        Ok(()) => 0,
        Err(error) => {
            tracing::warn!(error=%error,"virtual payment notification rejected");
            1
        }
    };
    let response = if xml {
        format!(
            "<xml><ErrCode>{code}</ErrCode><ErrMsg>{}</ErrMsg></xml>",
            if code == 0 { "success" } else { "retry" }
        )
    } else {
        serde_json::json!({"ErrCode":code,"ErrMsg":if code==0{"success"}else{"retry"}}).to_string()
    };
    (
        [(
            axum::http::header::CONTENT_TYPE,
            if xml {
                "application/xml"
            } else {
                "application/json"
            },
        )],
        response,
    )
}

pub async fn wechat_notify_handler(
    State(state): State<Arc<PaymentWebState>>,
    body: String,
) -> impl IntoResponse {
    let notify_data = parse_xml(&body);

    if state.wechat_pay_api_key.is_empty() {
        tracing::error!("wechat_pay_api_key is not configured, rejecting notify");
        return (
            StatusCode::OK,
            "<xml><return_code><![CDATA[FAIL]]></return_code></xml>",
        );
    }

    let original_sign = notify_data.get("sign").cloned().unwrap_or_default();
    if !verify_notify_sign(&notify_data, &state.wechat_pay_api_key, &original_sign) {
        tracing::warn!("wechat pay notify signature verification failed");
        return (
            StatusCode::OK,
            "<xml><return_code><![CDATA[FAIL]]></return_code></xml>",
        );
    }

    match state
        .handle_wechat_notify_use_case
        .execute(notify_data)
        .await
    {
        Ok(NotifyHandleResult::Success) => (
            StatusCode::OK,
            "<xml><return_code><![CDATA[SUCCESS]]></return_code><return_msg><![CDATA[OK]]></return_msg></xml>",
        ),
        Ok(NotifyHandleResult::ProtocolFailure) => (
            StatusCode::OK,
            "<xml><return_code><![CDATA[FAIL]]></return_code></xml>",
        ),
        Ok(NotifyHandleResult::OrderNotFound) => (
            StatusCode::OK,
            "<xml><return_code><![CDATA[FAIL]]></return_code></xml>",
        ),
        Ok(NotifyHandleResult::AmountMismatch) => (
            StatusCode::OK,
            "<xml><return_code><![CDATA[FAIL]]></return_code></xml>",
        ),
        Err(error) => {
            tracing::error!(error = %error, "failed to handle wechat notify");
            (
                StatusCode::OK,
                "<xml><return_code><![CDATA[FAIL]]></return_code></xml>",
            )
        }
    }
}

fn parse_xml(xml: &str) -> std::collections::HashMap<String, String> {
    let mut result = std::collections::HashMap::new();
    let mut rest = xml;

    while let Some(open_idx) = rest.find('<') {
        rest = &rest[open_idx + 1..];

        let Some(tag_end_idx) = rest.find('>') else {
            break;
        };

        let tag = rest[..tag_end_idx].trim();
        rest = &rest[tag_end_idx + 1..];

        if tag.is_empty() || tag.starts_with('/') || tag.starts_with('!') || tag.starts_with('?') {
            continue;
        }

        let close_tag = format!("</{tag}>");
        let Some(close_idx) = rest.find(&close_tag) else {
            continue;
        };

        let raw_value = &rest[..close_idx];
        if tag == "xml" {
            rest = raw_value;
            continue;
        }

        let value = raw_value
            .trim()
            .strip_prefix("<![CDATA[")
            .and_then(|item| item.strip_suffix("]]>"))
            .unwrap_or(raw_value.trim());

        result.insert(tag.to_string(), value.to_string());
        rest = &rest[close_idx + close_tag.len()..];
    }

    result
}

fn verify_notify_sign(
    data: &std::collections::HashMap<String, String>,
    api_key: &str,
    original_sign: &str,
) -> bool {
    let params: std::collections::BTreeMap<String, String> = data
        .iter()
        .filter(|(k, v)| k.as_str() != "sign" && !v.is_empty())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let sign_str = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&")
        + "&key="
        + api_key;
    let calculated = format!("{:x}", md5::compute(sign_str)).to_uppercase();
    calculated == original_sign
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use super::{parse_xml, verify_notify_sign};

    #[test]
    fn verify_notify_sign_ignores_empty_fields() {
        let api_key = "secret-key";
        let mut data = HashMap::new();
        data.insert("appid".to_string(), "wx-test".to_string());
        data.insert("mch_id".to_string(), "mch-test".to_string());
        data.insert("return_code".to_string(), "SUCCESS".to_string());
        data.insert("result_code".to_string(), "SUCCESS".to_string());
        data.insert("nonce_str".to_string(), "nonce-123".to_string());
        data.insert("coupon_type_0".to_string(), String::new());

        let sign = build_wechat_v2_sign(&data, api_key);

        assert!(verify_notify_sign(&data, api_key, &sign));
    }

    #[test]
    fn parse_xml_extracts_wechat_style_flat_fields() {
        let xml = "<xml><return_code><![CDATA[SUCCESS]]></return_code><result_code>SUCCESS</result_code><prepay_id><![CDATA[wx123]]></prepay_id></xml>";

        let parsed = parse_xml(xml);

        assert_eq!(
            parsed.get("return_code").map(String::as_str),
            Some("SUCCESS")
        );
        assert_eq!(
            parsed.get("result_code").map(String::as_str),
            Some("SUCCESS")
        );
        assert_eq!(parsed.get("prepay_id").map(String::as_str), Some("wx123"));
    }

    fn build_wechat_v2_sign(data: &HashMap<String, String>, api_key: &str) -> String {
        let params: BTreeMap<String, String> = data
            .iter()
            .filter(|(k, v)| k.as_str() != "sign" && !v.is_empty())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let sign_str = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
            + "&key="
            + api_key;

        format!("{:x}", md5::compute(sign_str)).to_uppercase()
    }
}
