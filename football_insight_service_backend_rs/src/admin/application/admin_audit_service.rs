use std::sync::Arc;

use crate::admin::{
    domain::{
        admin_audit::AdminAuditLogPage,
        admin_user::{normalize_page, normalize_page_size},
    },
    ports::admin_audit_repository::AdminAuditRepository,
};

pub struct AdminAuditService {
    repository: Arc<dyn AdminAuditRepository>,
}

impl AdminAuditService {
    pub fn new(repository: Arc<dyn AdminAuditRepository>) -> Self {
        Self { repository }
    }

    pub async fn list(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> anyhow::Result<AdminAuditLogPage> {
        self.repository
            .list(normalize_page(page), normalize_page_size(page_size))
            .await
    }
}
