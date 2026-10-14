use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum VirtualPaymentError {
    #[error("购买服务暂未开放，请稍后再试")]
    Unavailable,
    #[error("该商品暂不可购买，请稍后再试")]
    ProductUnavailable,
    #[error("商品配置暂不可用，请稍后再试")]
    CatalogUnavailable,
    #[error("请刷新微信支付会话后重试")]
    SessionExpired,
    #[error("当前微信身份与登录账号不一致，请重新登录")]
    IdentityMismatch,
    #[error("虚拟支付订单校验失败")]
    InvalidOrder,
    #[error("微信支付服务暂不可用，请稍后重试")]
    Provider,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VirtualProduct {
    #[serde(skip)]
    pub catalog_product_id: Option<uuid::Uuid>,
    pub product_id: String,
    pub price_cents: i32,
    pub membership_duration_months: i32,
}

#[derive(Debug, Clone)]
pub struct PrivateMembershipProduct {
    pub product_type: String,
    pub name: String,
    pub price_cents: i32,
    pub membership_duration_months: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualPayParams {
    pub mode: String,
    pub sign_data: String,
    pub pay_sig: String,
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct VirtualOrder {
    pub catalog_product_id: Option<uuid::Uuid>,
    pub order_no: String,
    pub open_id: String,
    pub product_id: String,
    pub amount_cents: i32,
    pub membership_duration_months: i32,
    pub wx_order_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Delivery {
    pub order_no: String,
    pub open_id: String,
    pub product_id: String,
    pub amount_cents: i32,
    pub original_price_cents: i32,
    pub quantity: i32,
    pub env: i32,
    pub wx_order_id: String,
}

#[derive(Debug, Clone)]
pub struct Refund {
    pub order_no: String,
    pub open_id: String,
    pub wx_order_id: String,
    pub refund_id: String,
    pub amount_cents: i32,
}
pub enum VirtualEvent {
    Delivery(Delivery),
    Refund(Refund),
    Ignored,
    RefundQuery,
}

pub fn product_key(product_type: &str, amount_cents: i32) -> Result<String, VirtualPaymentError> {
    if amount_cents <= 0 {
        return Err(VirtualPaymentError::InvalidOrder);
    }
    let parts: Vec<_> = product_type.split(':').collect();
    let key = match parts.as_slice() {
        ["membership", tier] if matches!(*tier, "V3" | "V6" | "V7" | "V8" | "V9") => {
            format!("membership:{tier}")
        }
        ["match_id_unlock", id] if id.parse::<i64>().is_ok_and(|v| v > 0) => {
            "match_id_unlock".into()
        }
        ["reflux_subscription", plan, team, id]
            if !plan.is_empty()
                && !team.is_empty()
                && (*id == "-" || id.parse::<i64>().is_ok_and(|v| v > 0)) =>
        {
            format!("reflux_subscription:{plan}:{team}")
        }
        _ => return Err(VirtualPaymentError::InvalidOrder),
    };
    Ok(format!("{key}@{amount_cents}"))
}

pub fn build_sign_data(
    offer_id: &str,
    order_no: &str,
    product: &VirtualProduct,
) -> Result<String, VirtualPaymentError> {
    if offer_id.is_empty()
        || !(8..=32).contains(&order_no.len())
        || order_no.starts_with('_')
        || !order_no
            .bytes()
            .all(|v| v.is_ascii_alphanumeric() || b"_-|*@".contains(&v))
        || product.product_id.is_empty()
        || product.price_cents <= 0
    {
        return Err(VirtualPaymentError::InvalidOrder);
    }
    Ok(serde_json::json!({"offerId":offer_id,"buyQuantity":1,"env":0,"currencyType":"CNY",
        "productId":product.product_id,"goodsPrice":product.price_cents,"outTradeNo":order_no,"attach":order_no}).to_string())
}

pub fn validate_delivery(
    order: &VirtualOrder,
    delivery: &Delivery,
) -> Result<(), VirtualPaymentError> {
    if order.order_no != delivery.order_no
        || order.open_id != delivery.open_id
        || order.product_id != delivery.product_id
        || order.amount_cents != delivery.amount_cents
        || order.amount_cents != delivery.original_price_cents
        || delivery.quantity != 1
        || delivery.env != 0
        || delivery.wx_order_id.is_empty()
        || order
            .wx_order_id
            .as_ref()
            .is_some_and(|id| id != &delivery.wx_order_id)
    {
        return Err(VirtualPaymentError::InvalidOrder);
    }
    Ok(())
}
