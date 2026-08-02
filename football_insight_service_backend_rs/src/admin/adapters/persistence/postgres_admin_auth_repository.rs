use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::admin::{
    domain::admin_auth::{AdminAccount, AdminBootstrapOwner, AdminSession},
    ports::admin_auth_repository::AdminAuthRepository,
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
            r#"
            INSERT INTO f_i_admin_users (
                id, username, password_hash, display_name, role, status
            ) VALUES ($1, $2, $3, $4, 'owner', 'active')
            ON CONFLICT (username) DO NOTHING
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&owner.username)
        .bind(&owner.password_hash)
        .bind(&owner.display_name)
        .execute(&mut *tx)
        .await?;

        let row = sqlx::query_as::<_, AdminAccountRow>(
            r#"
            SELECT id, username, password_hash, display_name, role, status, created_at, updated_at
              FROM f_i_admin_users
             WHERE username = $1
            "#,
        )
        .bind(owner.username)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(row.into())
    }
}
