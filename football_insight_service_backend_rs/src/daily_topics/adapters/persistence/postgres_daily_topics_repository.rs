use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use sqlx::{PgPool, Row};

use crate::daily_topics::{
    domain::edition::DailyTopicEdition,
    ports::daily_topics_repository::{DailyTopicsRepository, DailyTopicsRepositoryError},
};

use super::stored_payload::StoredPayload;

pub struct PostgresDailyTopicsRepository {
    pool: PgPool,
}

impl PostgresDailyTopicsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DailyTopicsRepository for PostgresDailyTopicsRepository {
    async fn latest_for_team(
        &self,
        team_id: i64,
    ) -> Result<Option<DailyTopicEdition>, DailyTopicsRepositoryError> {
        let row = sqlx::query(
            "SELECT team_id, edition_date, published_at, payload FROM f_i_daily_topic_editions WHERE team_id = $1 ORDER BY published_at DESC, id DESC LIMIT 1"
        ).bind(team_id).fetch_optional(&self.pool).await.map_err(|error| {
            tracing::error!(%error, team_id, "daily topics database query failed");
            DailyTopicsRepositoryError::Unavailable
        })?;
        let Some(row) = row else {
            return Ok(None);
        };
        // Metadata columns are authoritative. Do not let payload metadata override the selected team or edition.
        let team_id: i64 = row.try_get("team_id").map_err(invalid_row)?;
        let edition_date: NaiveDate = row.try_get("edition_date").map_err(invalid_row)?;
        let published_at: DateTime<Utc> = row.try_get("published_at").map_err(invalid_row)?;
        let payload: serde_json::Value = row.try_get("payload").map_err(invalid_row)?;
        let payload: StoredPayload = serde_json::from_value(payload).map_err(|error| {
            tracing::error!(%error, team_id, "invalid daily topics payload");
            DailyTopicsRepositoryError::InvalidEdition
        })?;
        let shanghai = FixedOffset::east_opt(8 * 3600).expect("valid Shanghai offset");
        Ok(Some(DailyTopicEdition {
            team_id,
            edition_date,
            published_at: published_at.with_timezone(&shanghai),
            items: payload.items.into_iter().map(Into::into).collect(),
        }))
    }
}

fn invalid_row(error: sqlx::Error) -> DailyTopicsRepositoryError {
    tracing::error!(%error, "invalid daily topics database row");
    DailyTopicsRepositoryError::InvalidEdition
}
