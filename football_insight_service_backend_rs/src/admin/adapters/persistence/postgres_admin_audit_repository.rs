use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::admin::{
    domain::admin_audit::{AdminAuditLog, AdminAuditLogPage},
    ports::admin_audit_repository::AdminAuditRepository,
};

#[derive(Clone)]
pub struct PostgresAdminAuditRepository {
    pool: PgPool,
}

impl PostgresAdminAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct AdminAuditRow {
    id: Uuid,
    admin_user_id: Uuid,
    admin_username: String,
    action: String,
    target_type: String,
    target_id: Option<String>,
    reason: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<AdminAuditRow> for AdminAuditLog {
    fn from(value: AdminAuditRow) -> Self {
        Self {
            id: value.id,
            admin_user_id: value.admin_user_id,
            admin_username: value.admin_username,
            action: value.action,
            target_type: value.target_type,
            target_id: value.target_id,
            reason: value.reason,
            created_at: value.created_at,
        }
    }
}

#[async_trait]
impl AdminAuditRepository for PostgresAdminAuditRepository {
    async fn list(&self, page: i64, page_size: i64) -> anyhow::Result<AdminAuditLogPage> {
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM f_i_admin_audit_logs")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, AdminAuditRow>(
            r#"
            SELECT logs.id,
                   logs.admin_user_id,
                   admins.username AS admin_username,
                   logs.action,
                   logs.target_type,
                   logs.target_id,
                   logs.reason,
                   logs.created_at
              FROM f_i_admin_audit_logs AS logs
              JOIN f_i_admin_users AS admins ON admins.id = logs.admin_user_id
             ORDER BY logs.created_at DESC, logs.id DESC
             LIMIT $1 OFFSET $2
            "#,
        )
        .bind(page_size)
        .bind((page - 1) * page_size)
        .fetch_all(&self.pool)
        .await?;
        Ok(AdminAuditLogPage {
            total,
            page,
            page_size,
            items: rows.into_iter().map(Into::into).collect(),
        })
    }
}
