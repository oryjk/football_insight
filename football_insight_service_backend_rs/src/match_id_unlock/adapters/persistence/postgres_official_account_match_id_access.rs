use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{
    auth::{
        domain::official_account_match_id_access::OfficialAccountMatchIdAccess,
        ports::official_account_match_id_access_port::OfficialAccountMatchIdAccessPort,
    },
    match_id_unlock::domain::official_account_access::{
        OfficialAccountUser, resolve_official_account_access_for_users,
    },
};

// CASE prevents the unlock subquery from running when the source ID is not numeric.
const ACCESS_SQL: &str = r#"
SELECT u.membership_tier, u.membership_expires_at,
       CASE WHEN $2::bigint IS NULL THEN false
            ELSE EXISTS (
                SELECT 1 FROM f_i_user_match_id_unlocks x
                WHERE x.user_id = u.id AND x.match_id = $2
            ) END AS has_unlock_for_match
FROM f_i_users u
WHERE u.official_wechat_open_id = $1 AND u.status = 'active'
"#;
// No LIMIT: the openid is not unique, so every linked active account is considered (see the domain fn).

pub struct PostgresOfficialAccountMatchIdAccess {
    pool: PgPool,
}

impl PostgresOfficialAccountMatchIdAccess {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OfficialAccountMatchIdAccessPort for PostgresOfficialAccountMatchIdAccess {
    async fn check(
        &self,
        official_open_id: &str,
        match_id: Option<i64>,
    ) -> anyhow::Result<OfficialAccountMatchIdAccess> {
        let rows = sqlx::query_as::<_, (String, Option<DateTime<Utc>>, bool)>(ACCESS_SQL)
            .bind(official_open_id)
            .bind(match_id)
            .fetch_all(&self.pool)
            .await?;
        let users: Vec<OfficialAccountUser> = rows
            .into_iter()
            .map(
                |(membership_tier, membership_expires_at, has_unlock_for_match)| {
                    OfficialAccountUser {
                        membership_tier,
                        membership_expires_at,
                        has_unlock_for_match,
                    }
                },
            )
            .collect();
        Ok(resolve_official_account_access_for_users(
            &users,
            Utc::now(),
        ))
    }

    async fn record_acquisition(
        &self,
        official_open_id: &str,
        require_first: bool,
    ) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;
        // Same lock as official binding and invited registration, always before account rows.
        // Serializing these short writes prevents a binding change from hiding acquisition history.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended('football_insight:official-match-id-history', 0))")
            .execute(&mut *tx)
            .await?;
        // Lock linked accounts before testing history, including inactive accounts. This
        // also coordinates with binding changes and records all accounts sharing an openid.
        let histories = sqlx::query_scalar::<_, bool>(
            "SELECT official_match_id_received FROM f_i_users
             WHERE official_wechat_open_id = $1 ORDER BY id FOR UPDATE",
        )
        .bind(official_open_id)
        .fetch_all(&mut *tx)
        .await?;
        if require_first && histories.iter().any(|received| *received) {
            return Ok(false);
        }
        // The unique key arbitrates simultaneous requests even for unbound followers.
        let inserted = sqlx::query_scalar::<_, String>(
            "INSERT INTO f_i_official_match_id_acquisitions (official_open_id, source)
             VALUES ($1, 'acquired') ON CONFLICT (official_open_id) DO NOTHING
             RETURNING official_open_id",
        )
        .bind(official_open_id)
        .fetch_optional(&mut *tx)
        .await?;
        let granted = !require_first || inserted.is_some();
        sqlx::query(
            "UPDATE f_i_users SET official_match_id_received = TRUE
             WHERE official_wechat_open_id = $1 AND NOT official_match_id_received",
        )
        .bind(official_open_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(granted)
    }
}
