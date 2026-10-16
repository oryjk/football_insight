use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PaymentRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub scenario: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AdminPaymentError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("payment observability query failed")]
    Internal,
}

#[derive(Debug, Clone)]
pub struct FunnelCount {
    pub day: NaiveDate,
    pub scenario: String,
    pub product_ref: Option<String>,
    pub event: String,
    pub attempt_count: i64,
    pub user_count: i64,
    pub previous_event: Option<String>,
    pub previous_attempt_count: Option<i64>,
    pub converted_attempt_count: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct FunnelStage {
    pub counts: FunnelCount,
    pub conversion_rate: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AttemptTerminalCount {
    pub day: NaiveDate,
    pub scenario: String,
    pub product_ref: Option<String>,
    pub events: Vec<String>,
    pub attempt_count: i64,
    pub user_ids: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct TerminalCount {
    pub day: NaiveDate,
    pub scenario: String,
    pub product_ref: Option<String>,
    pub stage: String,
    pub attempt_count: i64,
    pub user_count: i64,
}

#[derive(Debug, Clone)]
pub struct PaymentErrorCount {
    pub event: String,
    pub error_code: Option<String>,
    pub error_msg: Option<String>,
    pub count: i64,
    pub total_error_count: i64,
    pub last_occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PaymentTimelineEvent {
    pub id: i64,
    pub attempt_id: Uuid,
    pub order_no: Option<String>,
    pub user_id: Uuid,
    pub scenario: String,
    pub product_ref: Option<String>,
    pub event: String,
    pub source: String,
    pub error_code: Option<String>,
    pub error_msg: Option<String>,
    pub detail: serde_json::Value,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct StuckPaymentOrder {
    pub order_no: String,
    pub user_id: Uuid,
    pub product_type: String,
    pub created_at: DateTime<Utc>,
    pub last_event: Option<String>,
    pub last_event_at: Option<DateTime<Utc>>,
    pub events: Vec<String>,
}

pub struct StuckPaymentStage {
    pub order: StuckPaymentOrder,
    pub stage: String,
}

pub struct PaymentFunnel {
    pub stages: Vec<FunnelStage>,
    pub terminal_stages: Vec<TerminalCount>,
}
