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
                |(membership_tier, membership_expires_at, has_unlock_for_match)| OfficialAccountUser {
                    membership_tier,
                    membership_expires_at,
                    has_unlock_for_match,
                },
            )
            .collect();
        Ok(resolve_official_account_access_for_users(&users, Utc::now()))
    }
}
