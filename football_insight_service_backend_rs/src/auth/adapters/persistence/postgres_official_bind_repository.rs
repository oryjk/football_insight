use crate::auth::{
    domain::official_bind::{ConsumeBindCodeOutcome, IssueBindCodeOutcome, can_issue_bind_code},
    ports::official_bind_repository::OfficialBindRepository,
};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

const LOCK_ACTIVE_USER_SQL: &str =
    r#"SELECT id FROM f_i_users WHERE id=$1 AND status='active' FOR UPDATE"#;
const COUNT_CODES_SQL: &str =
    r#"SELECT COUNT(*) FROM f_i_wechat_official_bind_codes WHERE user_id=$1 AND created_at >= $2"#;
const INVALIDATE_CODES_SQL: &str = r#"UPDATE f_i_wechat_official_bind_codes SET used_at=$2 WHERE user_id=$1 AND used_at IS NULL AND expires_at>$2"#;
const INSERT_CODE_SQL: &str = r#"INSERT INTO f_i_wechat_official_bind_codes (user_id,code_hash,expires_at,created_at) VALUES ($1,$2,$3,$4)"#;
const FIND_CODE_USER_SQL: &str =
    r#"SELECT user_id FROM f_i_wechat_official_bind_codes WHERE code_hash=$1"#;
const LOCK_BIND_USER_SQL: &str =
    r#"SELECT official_wechat_open_id FROM f_i_users WHERE id=$1 AND status='active' FOR UPDATE"#;
const CONSUME_CODE_SQL: &str = r#"UPDATE f_i_wechat_official_bind_codes SET used_at=NOW() WHERE code_hash=$1 AND used_at IS NULL AND expires_at>statement_timestamp() RETURNING user_id"#;
const UPDATE_OFFICIAL_SQL: &str = r#"
UPDATE f_i_users
SET official_wechat_open_id=$2,
    official_match_id_received = official_match_id_received OR EXISTS (
        SELECT 1 FROM f_i_official_match_id_acquisitions
        WHERE official_open_id=$2 OR official_open_id=f_i_users.official_wechat_open_id
    )
WHERE id=$1 AND status='active'
"#;

pub struct PostgresOfficialBindRepository {
    pool: PgPool,
}
impl PostgresOfficialBindRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OfficialBindRepository for PostgresOfficialBindRepository {
    async fn issue(
        &self,
        user_id: Uuid,
        code_hash: &str,
        now: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> anyhow::Result<IssueBindCodeOutcome> {
        let mut tx = self.pool.begin().await?;
        // Issue and consume always lock the user BEFORE the code to avoid lock inversion.
        if sqlx::query_scalar::<_, Uuid>(LOCK_ACTIVE_USER_SQL)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
            .is_none()
        {
            tx.rollback().await?;
            return Ok(IssueBindCodeOutcome::Inactive);
        }
        let count = sqlx::query_scalar::<_, i64>(COUNT_CODES_SQL)
            .bind(user_id)
            .bind(now - Duration::hours(1))
            .fetch_one(&mut *tx)
            .await?;
        if !can_issue_bind_code(count) {
            tx.rollback().await?;
            return Ok(IssueBindCodeOutcome::RateLimited);
        }
        sqlx::query(INVALIDATE_CODES_SQL)
            .bind(user_id)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        let insert = sqlx::query(INSERT_CODE_SQL)
            .bind(user_id)
            .bind(code_hash)
            .bind(expires_at)
            .bind(now)
            .execute(&mut *tx)
            .await;
        if let Err(error) = insert {
            let collision = error
                .as_database_error()
                .is_some_and(|e| e.code().as_deref() == Some("23505"));
            tx.rollback().await?;
            if collision {
                return Ok(IssueBindCodeOutcome::Collision);
            }
            return Err(error.into());
        }
        tx.commit().await?;
        Ok(IssueBindCodeOutcome::Created)
    }

    async fn consume(
        &self,
        code_hash: &str,
        verified_official_open_id: &str,
    ) -> anyhow::Result<ConsumeBindCodeOutcome> {
        // Only called after verified webhook parsing. Reject empty/invalid source identities defensively.
        if verified_official_open_id.trim().is_empty()
            || verified_official_open_id.trim() != verified_official_open_id
            || verified_official_open_id.len() > 128
        {
            return Ok(ConsumeBindCodeOutcome::Invalid);
        }
        let mut tx = self.pool.begin().await?;
        // Acquire the shared history lock before user/code row locks, as acquisition and
        // invited registration do, so a concurrent acquisition cannot lose its account binding.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('football_insight:official-match-id-history', 0))")
            .execute(&mut *tx)
            .await?;
        let Some(user_id) = sqlx::query_scalar::<_, Uuid>(FIND_CODE_USER_SQL)
            .bind(code_hash)
            .fetch_optional(&mut *tx)
            .await?
        else {
            tx.rollback().await?;
            return Ok(ConsumeBindCodeOutcome::Invalid);
        };
        let Some(current_open_id) = sqlx::query_scalar::<_, Option<String>>(LOCK_BIND_USER_SQL)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?
        else {
            tx.rollback().await?;
            return Ok(ConsumeBindCodeOutcome::Invalid);
        };
        if sqlx::query_scalar::<_, Uuid>(CONSUME_CODE_SQL)
            .bind(code_hash)
            .fetch_optional(&mut *tx)
            .await?
            .is_none()
        {
            tx.rollback().await?;
            return Ok(ConsumeBindCodeOutcome::Invalid);
        }
        let already_bound = current_open_id.as_deref() == Some(verified_official_open_id);
        if !already_bound {
            let result = sqlx::query(UPDATE_OFFICIAL_SQL)
                .bind(user_id)
                .bind(verified_official_open_id)
                .execute(&mut *tx)
                .await?;
            if result.rows_affected() != 1 {
                tx.rollback().await?;
                return Ok(ConsumeBindCodeOutcome::Invalid);
            }
        }
        tx.commit().await?;
        Ok(if already_bound {
            ConsumeBindCodeOutcome::AlreadyBound
        } else {
            ConsumeBindCodeOutcome::Success
        })
    }
}
