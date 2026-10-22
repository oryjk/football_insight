use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::admin::{
    domain::admin_paywall::{AdminPaywallError, PaywallFunnelRow, PaywallRange, PaywallTierRow},
    ports::admin_paywall_repository::AdminPaywallRepository,
};

pub struct PostgresAdminPaywallRepository {
    pool: PgPool,
}

impl PostgresAdminPaywallRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Days are Beijing calendar days, not UTC, so a "match day" reads the way operators expect.
pub const FUNNEL_SQL: &str = r#"
SELECT (occurred_at AT TIME ZONE 'Asia/Shanghai')::date AS day, surface,
       COUNT(*) FILTER (WHERE action='impression')::bigint AS impressions,
       COUNT(DISTINCT user_id) FILTER (WHERE action='impression')::bigint AS impression_users,
       COUNT(*) FILTER (WHERE action='tap')::bigint AS taps,
       COUNT(DISTINCT user_id) FILTER (WHERE action='tap')::bigint AS tap_users
FROM f_i_paywall_events
WHERE occurred_at >= $1 AND occurred_at < $2 AND ($3::text IS NULL OR surface = $3)
GROUP BY 1, 2 ORDER BY 1, 2
"#;

pub const BY_TIER_SQL: &str = r#"
SELECT membership_tier, surface,
       COUNT(DISTINCT user_id) FILTER (WHERE action='impression')::bigint AS impression_users,
       COUNT(DISTINCT user_id) FILTER (WHERE action='tap')::bigint AS tap_users
FROM f_i_paywall_events
WHERE occurred_at >= $1 AND occurred_at < $2 AND ($3::text IS NULL OR surface = $3)
GROUP BY 1, 2 ORDER BY 1 NULLS LAST, 2
"#;

fn map_error(error: sqlx::Error) -> AdminPaywallError {
    tracing::warn!(%error, "admin paywall measurement query failed");
    AdminPaywallError::Internal
}

#[async_trait]
impl AdminPaywallRepository for PostgresAdminPaywallRepository {
    async fn funnel(
        &self,
        range: &PaywallRange,
    ) -> Result<Vec<PaywallFunnelRow>, AdminPaywallError> {
        let rows = sqlx::query(FUNNEL_SQL)
            .bind(range.from)
            .bind(range.to)
            .bind(&range.surface)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.iter()
            .map(|row| {
                Ok(PaywallFunnelRow {
                    day: row.try_get("day")?,
                    surface: row.try_get("surface")?,
                    impressions: row.try_get("impressions")?,
                    impression_users: row.try_get("impression_users")?,
                    taps: row.try_get("taps")?,
                    tap_users: row.try_get("tap_users")?,
                })
            })
            .collect::<Result<_, sqlx::Error>>()
            .map_err(map_error)
    }

    async fn by_tier(
        &self,
        range: &PaywallRange,
    ) -> Result<Vec<PaywallTierRow>, AdminPaywallError> {
        let rows = sqlx::query(BY_TIER_SQL)
            .bind(range.from)
            .bind(range.to)
            .bind(&range.surface)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.iter()
            .map(|row| {
                Ok(PaywallTierRow {
                    membership_tier: row.try_get("membership_tier")?,
                    surface: row.try_get("surface")?,
                    impression_users: row.try_get("impression_users")?,
                    tap_users: row.try_get("tap_users")?,
                })
            })
            .collect::<Result<_, sqlx::Error>>()
            .map_err(map_error)
    }
}
