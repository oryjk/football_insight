use crate::admin::{
    domain::admin_credentials::{
        AdminCredentialsChange, AdminCredentialsError, AdminCredentialsSnapshot,
    },
    ports::admin_credentials_repository::AdminCredentialsRepository,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
pub struct PostgresAdminCredentialsRepository {
    pool: PgPool,
}
impl PostgresAdminCredentialsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl AdminCredentialsRepository for PostgresAdminCredentialsRepository {
    async fn load_for_session(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
    ) -> Result<AdminCredentialsSnapshot, AdminCredentialsError> {
        let row=sqlx::query_as::<_,(String,String)>("SELECT users.username,users.password_hash FROM f_i_admin_users users JOIN f_i_admin_sessions sessions ON sessions.admin_user_id=users.id WHERE users.id=$1 AND users.status='active' AND sessions.id=$2 AND sessions.revoked_at IS NULL AND sessions.expires_at>clock_timestamp()")
            .bind(admin_id).bind(session_id).fetch_optional(&self.pool).await.map_err(map_error)?.ok_or(AdminCredentialsError::Unauthorized)?;
        Ok(AdminCredentialsSnapshot {
            username: row.0,
            password_hash: row.1,
        })
    }
    async fn change_with_audit(
        &self,
        change: AdminCredentialsChange,
    ) -> Result<(), AdminCredentialsError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let account = sqlx::query_as::<_, (String, String, String)>(
            "SELECT username,password_hash,status FROM f_i_admin_users WHERE id=$1 FOR UPDATE",
        )
        .bind(change.admin_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_error)?
        .ok_or(AdminCredentialsError::Unauthorized)?;
        if account.2 != "active"
            || account.0 != change.expected_username
            || account.1 != change.expected_password_hash
        {
            return Err(AdminCredentialsError::Unauthorized);
        }
        let active=sqlx::query_scalar::<_,Uuid>("SELECT id FROM f_i_admin_sessions WHERE admin_user_id=$1 AND id=$2 AND revoked_at IS NULL AND expires_at>clock_timestamp() FOR UPDATE")
            .bind(change.admin_id).bind(change.session_id).fetch_optional(&mut *tx).await.map_err(map_error)?;
        if active.is_none() {
            return Err(AdminCredentialsError::Unauthorized);
        }
        sqlx::query("UPDATE f_i_admin_users SET username=$2,password_hash=COALESCE($3,password_hash),updated_at=clock_timestamp() WHERE id=$1")
            .bind(change.admin_id).bind(&change.username).bind(&change.password_hash).execute(&mut *tx).await.map_err(map_error)?;
        sqlx::query("UPDATE f_i_admin_sessions SET revoked_at=clock_timestamp() WHERE admin_user_id=$1 AND revoked_at IS NULL")
            .bind(change.admin_id).execute(&mut *tx).await.map_err(map_error)?;
        let before = serde_json::json!({"username":account.0});
        let after = serde_json::json!({"username":change.username,"password_changed":change.password_hash.is_some()});
        sqlx::query("INSERT INTO f_i_admin_audit_logs(id,admin_user_id,action,target_type,target_id,before_json,after_json) VALUES($1,$2,'admin.credentials.update','admin_account',$3,$4,$5)")
            .bind(Uuid::new_v4()).bind(change.admin_id).bind(change.admin_id.to_string()).bind(before).bind(after).execute(&mut *tx).await.map_err(map_error)?;
        tx.commit().await.map_err(map_error)?;
        Ok(())
    }
}
fn map_error(error: sqlx::Error) -> AdminCredentialsError {
    if error
        .as_database_error()
        .is_some_and(|error| error.is_unique_violation())
    {
        return AdminCredentialsError::Conflict;
    }
    // Database errors can include a failed row containing a hash. Never log their detail here.
    tracing::error!("admin credentials persistence failed");
    AdminCredentialsError::Internal
}
