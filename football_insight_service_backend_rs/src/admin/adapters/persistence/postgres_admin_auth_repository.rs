use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::admin::{
    domain::{
        admin_auth::{AdminAccount, AdminBootstrapOwner, AdminSession},
        admin_setup::AdminSetupError,
    },
    ports::{
        admin_auth_repository::AdminAuthRepository, admin_setup_repository::AdminSetupRepository,
    },
};

#[derive(Clone)]
pub struct PostgresAdminAuthRepository {
    pool: PgPool,
}

impl PostgresAdminAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct AdminAccountRow {
    id: Uuid,
    username: String,
    password_hash: String,
    display_name: String,
    role: String,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<AdminAccountRow> for AdminAccount {
    fn from(value: AdminAccountRow) -> Self {
        Self {
            id: value.id,
            username: value.username,
            password_hash: value.password_hash,
            display_name: value.display_name,
            role: value.role,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[async_trait]
impl AdminAuthRepository for PostgresAdminAuthRepository {
    async fn find_by_username(&self, username: &str) -> anyhow::Result<Option<AdminAccount>> {
        let row = sqlx::query_as::<_, AdminAccountRow>(
            r#"
            SELECT id, username, password_hash, display_name, role, status, created_at, updated_at
              FROM f_i_admin_users
             WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn create_session(&self, session: AdminSession) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO f_i_admin_sessions (
                id, admin_user_id, expires_at, revoked_at, created_at, last_seen_at
            ) VALUES ($1, $2, $3, $4, $5, $5)
            "#,
        )
        .bind(session.id)
        .bind(session.admin_user_id)
        .bind(session.expires_at)
        .bind(session.revoked_at)
        .bind(session.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn create_session_if_current(
        &self,
        session: AdminSession,
        expected_username: &str,
        expected_password_hash: &str,
    ) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;
        let current=sqlx::query_scalar::<_,Uuid>("SELECT id FROM f_i_admin_users WHERE id=$1 AND username=$2 AND password_hash=$3 AND status='active' AND $4::timestamptz>clock_timestamp() FOR UPDATE")
            .bind(session.admin_user_id).bind(expected_username).bind(expected_password_hash).bind(session.expires_at).fetch_optional(&mut *tx).await?;
        if current.is_none() {
            return Ok(false);
        }
        sqlx::query("INSERT INTO f_i_admin_sessions(id,admin_user_id,expires_at,revoked_at,created_at,last_seen_at) VALUES($1,$2,$3,$4,$5,$5)")
            .bind(session.id).bind(session.admin_user_id).bind(session.expires_at).bind(session.revoked_at).bind(session.created_at).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(true)
    }

    async fn find_active_account_for_session(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Option<AdminAccount>> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, AdminAccountRow>(
            r#"
            SELECT users.id,
                   users.username,
                   users.password_hash,
                   users.display_name,
                   users.role,
                   users.status,
                   users.created_at,
                   users.updated_at
              FROM f_i_admin_sessions AS sessions
              JOIN f_i_admin_users AS users ON users.id = sessions.admin_user_id
             WHERE sessions.id = $1
               AND sessions.admin_user_id = $2
               AND sessions.revoked_at IS NULL
               AND sessions.expires_at > $3
               AND users.status = 'active'
            "#,
        )
        .bind(session_id)
        .bind(admin_id)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await?;

        if row.is_some() {
            sqlx::query("UPDATE f_i_admin_sessions SET last_seen_at = $2 WHERE id = $1")
                .bind(session_id)
                .bind(now)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(row.map(Into::into))
    }

    async fn revoke_session(
        &self,
        admin_id: Uuid,
        session_id: Uuid,
        revoked_at: DateTime<Utc>,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE f_i_admin_sessions
               SET revoked_at = $3
             WHERE id = $1
               AND admin_user_id = $2
               AND revoked_at IS NULL
            "#,
        )
        .bind(session_id)
        .bind(admin_id)
        .bind(revoked_at)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn ensure_owner(&self, owner: AdminBootstrapOwner) -> anyhow::Result<AdminAccount> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "SELECT pg_advisory_xact_lock(hashtext('football_insight_admin_owner_bootstrap'))",
        )
        .execute(&mut *tx)
        .await?;
        let existing=sqlx::query_as::<_,AdminAccountRow>("SELECT id,username,password_hash,display_name,role,status,created_at,updated_at FROM f_i_admin_users ORDER BY (role='owner') DESC,created_at,id LIMIT 1 FOR UPDATE").fetch_optional(&mut *tx).await?;
        if let Some(existing) = existing {
            tx.commit().await?;
            return Ok(existing.into());
        }
        sqlx::query("INSERT INTO f_i_admin_users(id,username,password_hash,display_name,role,status) VALUES($1,$2,$3,$4,'owner','active') ON CONFLICT(username) DO NOTHING")
            .bind(Uuid::new_v4()).bind(&owner.username).bind(&owner.password_hash).bind(&owner.display_name).execute(&mut *tx).await?;
        let row=sqlx::query_as::<_,AdminAccountRow>("SELECT id,username,password_hash,display_name,role,status,created_at,updated_at FROM f_i_admin_users WHERE username=$1").bind(&owner.username).fetch_one(&mut *tx).await?;
        if row.role != "owner" {
            anyhow::bail!("bootstrap username belongs to a non-owner account");
        }
        tx.commit().await?;
        Ok(row.into())
    }
}

#[async_trait]
impl AdminSetupRepository for PostgresAdminAuthRepository {
    async fn has_any_admin(&self) -> Result<bool, AdminSetupError> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM f_i_admin_users)")
            .fetch_one(&self.pool)
            .await
            .map_err(map_setup_error)
    }
    async fn initialize_superadmin(&self, password_hash: String) -> Result<(), AdminSetupError> {
        let mut tx = self.pool.begin().await.map_err(map_setup_error)?;
        sqlx::query(
            "SELECT pg_advisory_xact_lock(hashtext('football_insight_admin_owner_bootstrap'))",
        )
        .execute(&mut *tx)
        .await
        .map_err(map_setup_error)?;
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM f_i_admin_users)")
            .fetch_one(&mut *tx)
            .await
            .map_err(map_setup_error)?;
        if exists {
            return Err(AdminSetupError::AlreadyInitialized);
        }
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO f_i_admin_users(id,username,password_hash,display_name,role,status) VALUES($1,'superadmin',$2,'Super Admin','owner','active')").bind(id).bind(password_hash).execute(&mut *tx).await.map_err(map_setup_error)?;
        sqlx::query("INSERT INTO f_i_admin_audit_logs(id,admin_user_id,action,target_type,target_id,after_json) VALUES($1,$2,'admin.initialize','admin_account',$3,$4)").bind(Uuid::new_v4()).bind(id).bind(id.to_string()).bind(serde_json::json!({"username":"superadmin"})).execute(&mut *tx).await.map_err(map_setup_error)?;
        tx.commit().await.map_err(map_setup_error)?;
        Ok(())
    }
}
fn map_setup_error(error: sqlx::Error) -> AdminSetupError {
    if error
        .as_database_error()
        .is_some_and(|error| error.is_unique_violation())
    {
        return AdminSetupError::AlreadyInitialized;
    }
    tracing::error!("admin initialization persistence failed");
    AdminSetupError::Internal
}
