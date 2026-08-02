use std::sync::Arc;

use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use football_insight_service_backend_rs::admin::{
    application::admin_audit_service::AdminAuditService,
    domain::admin_audit::{AdminAuditLog, AdminAuditLogPage},
    ports::admin_audit_repository::AdminAuditRepository,
};
use uuid::Uuid;

struct FakeAuditRepository;

#[async_trait]
impl AdminAuditRepository for FakeAuditRepository {
    async fn list(&self, page: i64, page_size: i64) -> anyhow::Result<AdminAuditLogPage> {
        Ok(AdminAuditLogPage {
            total: 1,
            page,
            page_size,
            items: vec![AdminAuditLog {
                id: Uuid::new_v4(),
                admin_user_id: Uuid::new_v4(),
                admin_username: "owner".to_string(),
                action: "user.membership.adjust".to_string(),
                target_type: "user".to_string(),
                target_id: Some("user-id".to_string()),
                reason: Some("年度会员".to_string()),
                created_at: Utc.with_ymd_and_hms(2026, 8, 2, 12, 0, 0).unwrap(),
            }],
        })
    }
}

#[tokio::test]
async fn audit_service_normalizes_pagination_and_returns_entries() {
    let service = AdminAuditService::new(Arc::new(FakeAuditRepository));
    let result = service.list(Some(0), Some(500)).await.unwrap();
    assert_eq!(result.page, 1);
    assert_eq!(result.page_size, 100);
    assert_eq!(result.items[0].admin_username, "owner");
}
