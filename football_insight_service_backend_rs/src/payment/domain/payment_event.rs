use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentEventKind {
    PurchaseTapped,
    AgreementPrompted,
    SessionPrepareFailed,
    OrderCreateRequested,
    OrderCreateFailed,
    OrderCreated,
    PayParamsFailed,
    PayParamsIssued,
    PaySheetInvoked,
    PayCancelled,
    PayFailed,
    PaySuccessClient,
    ConfirmTimeout,
    Delivered,
    Refunded,
}

impl PaymentEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PurchaseTapped => "purchase_tapped",
            Self::AgreementPrompted => "agreement_prompted",
            Self::SessionPrepareFailed => "session_prepare_failed",
            Self::OrderCreateRequested => "order_create_requested",
            Self::OrderCreateFailed => "order_create_failed",
            Self::OrderCreated => "order_created",
            Self::PayParamsFailed => "pay_params_failed",
            Self::PayParamsIssued => "pay_params_issued",
            Self::PaySheetInvoked => "pay_sheet_invoked",
            Self::PayCancelled => "pay_cancelled",
            Self::PayFailed => "pay_failed",
            Self::PaySuccessClient => "pay_success_client",
            Self::ConfirmTimeout => "confirm_timeout",
            Self::Delivered => "delivered",
            Self::Refunded => "refunded",
        }
    }
    pub fn from_client_str(value: &str) -> Option<Self> {
        Some(match value {
            "purchase_tapped" => Self::PurchaseTapped,
            "agreement_prompted" => Self::AgreementPrompted,
            "session_prepare_failed" => Self::SessionPrepareFailed,
            "order_create_requested" => Self::OrderCreateRequested,
            "order_create_failed" => Self::OrderCreateFailed,
            "pay_sheet_invoked" => Self::PaySheetInvoked,
            "pay_cancelled" => Self::PayCancelled,
            "pay_failed" => Self::PayFailed,
            "pay_success_client" => Self::PaySuccessClient,
            "confirm_timeout" => Self::ConfirmTimeout,
            _ => return None,
        })
    }
    pub fn source(self) -> &'static str {
        match self {
            Self::OrderCreated
            | Self::PayParamsFailed
            | Self::PayParamsIssued
            | Self::Delivered
            | Self::Refunded => "server",
            _ => "client",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewPaymentEvent {
    pub attempt_id: Uuid,
    pub order_no: Option<String>,
    pub user_id: Uuid,
    pub scenario: String,
    pub product_ref: Option<String>,
    pub event: PaymentEventKind,
    pub error_code: Option<String>,
    pub error_msg: Option<String>,
    pub detail: Value,
    pub occurred_at: DateTime<Utc>,
}

pub fn attempt_id(header: Option<&str>) -> Uuid {
    header
        .and_then(|v| Uuid::parse_str(v).ok())
        .filter(|id| !id.is_nil())
        .unwrap_or_else(Uuid::new_v4)
}

pub fn valid_scenario(scenario: &str) -> bool {
    matches!(
        scenario,
        "membership" | "match_id_unlock" | "reflux_subscription"
    )
}

pub fn sanitize_message(value: &str) -> String {
    value.chars().take(300).collect()
}

pub fn sanitize_detail(value: &Value) -> Value {
    let mut output = Map::new();
    for key in [
        "mini_version",
        "wx_lib_version",
        "platform",
        "system",
        "brand",
        "model",
        "network_type",
        "elapsed_ms",
        "http_status",
    ] {
        let Some(value) = value.get(key) else {
            continue;
        };
        // Scalars only: nested objects could smuggle sensitive keys through the whitelist.
        let valid = if matches!(key, "elapsed_ms" | "http_status") {
            value.is_number()
        } else {
            value.is_string()
        };
        if !valid {
            continue;
        }
        output.insert(key.into(), value.clone());
        if serde_json::to_vec(&output).map_or(true, |v| v.len() > 1024) {
            output.remove(key);
        }
    }
    Value::Object(output)
}

/// Rank observed progress so delayed/reordered client batches cannot erase terminal evidence.
/// Provider refund/delivery is authoritative over all client reports.
pub fn terminal_stage(events: &[String]) -> &'static str {
    events
        .iter()
        .map(|event| match event.as_str() {
            "refunded" => (12, "refunded"),
            "delivered" => (11, "success"),
            "confirm_timeout" => (10, "confirmation_timeout"),
            "pay_success_client" => (9, "paid_awaiting_confirmation"),
            "pay_cancelled" => (8, "cancelled"),
            "pay_failed" => (8, "payment_failed"),
            "order_create_failed" | "pay_params_failed" => (7, "order_failed"),
            "pay_sheet_invoked" => (6, "pay_sheet_without_callback"),
            "pay_params_issued" => (5, "params_without_pay_sheet"),
            "session_prepare_failed" => (4, "session_prepare_failed"),
            "agreement_prompted" => (3, "agreement_not_accepted"),
            _ => (0, "unknown"),
        })
        .max_by_key(|(rank, _)| *rank)
        .map_or("unknown", |(_, stage)| stage)
}
