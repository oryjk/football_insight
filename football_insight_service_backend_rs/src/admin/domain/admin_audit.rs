use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAuditLog {
    pub id: Uuid,
    pub admin_user_id: Uuid,
    pub admin_username: String,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAuditLogPage {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub items: Vec<AdminAuditLog>,
}
