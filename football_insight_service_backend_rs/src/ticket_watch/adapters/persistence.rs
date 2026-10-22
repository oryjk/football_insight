use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use sqlx::{FromRow, PgPool};

use crate::ticket_watch::{
    domain::best_watch_windows::{
        RESTOCK_PROFILE_SCHEMA_VERSION, RESTOCK_PROFILE_STAT_UNIT, RestockProfile,
    },
    ports::restock_profile_store::RestockProfileStore,
};

pub struct PostgresRestockProfileStore {
    pool: PgPool,
}

impl PostgresRestockProfileStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct Row {
    total_events: i64,
    appear_events: i64,
    up_events: i64,
    seats_with_events: i64,
    hourly: Vec<i64>,
}

fn to_u64(value: i64) -> anyhow::Result<u64> {
    u64::try_from(value).map_err(|_| anyhow::anyhow!("negative restock count"))
}

#[async_trait]
impl RestockProfileStore for PostgresRestockProfileStore {
    async fn load(
        &self,
        lookup_ids: &[i64],
        since: DateTime<FixedOffset>,
    ) -> anyhow::Result<Option<RestockProfile>> {
        // 按候选 id 的先后顺序取第一条有事件的；全是空快照时退回第一条空的。
        let mut empty = None;
        for id in lookup_ids {
            let row = sqlx::query_as::<_, Row>(
                "SELECT total_events, appear_events, up_events, seats_with_events, hourly
                 FROM f_i_ticket_restock_profiles
                 WHERE match_id = $1 AND schema_version = $2 AND stat_unit = $3 AND since_at = $4",
            )
            .bind(id)
            .bind(i16::from(RESTOCK_PROFILE_SCHEMA_VERSION))
            .bind(RESTOCK_PROFILE_STAT_UNIT)
            .bind(since)
            .fetch_optional(&self.pool)
            .await?;
            let Some(row) = row else { continue };
            let hourly: [u64; 24] = row
                .hourly
                .iter()
                .map(|value| to_u64(*value))
                .collect::<anyhow::Result<Vec<_>>>()?
                .try_into()
                .map_err(|_| anyhow::anyhow!("restock hourly must have 24 buckets"))?;
            let profile = RestockProfile {
                total_events: to_u64(row.total_events)?,
                appear_events: to_u64(row.appear_events)?,
                up_events: to_u64(row.up_events)?,
                seats_with_events: to_u64(row.seats_with_events)?,
                hourly,
            };
            if profile.total_events > 0 {
                return Ok(Some(profile));
            }
            empty = Some(profile);
        }
        Ok(empty)
    }
}
