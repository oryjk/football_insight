use async_trait::async_trait;

use crate::admin::domain::admin_audit::AdminAuditLogPage;

#[async_trait]
pub trait AdminAuditRepository: Send + Sync {
    async fn list(&self, page: i64, page_size: i64) -> anyhow::Result<AdminAuditLogPage>;
}
