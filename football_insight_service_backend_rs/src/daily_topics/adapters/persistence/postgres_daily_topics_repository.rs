use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use sqlx::{PgPool, Row};

use crate::daily_topics::{
    domain::{
        archive::TopicPosition,
        edition::{DailyTopic, DailyTopicEdition},
    },
    ports::daily_topics_repository::{DailyTopicsRepository, DailyTopicsRepositoryError},
};

use super::stored_payload::{StoredPayload, StoredTopic};

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

    async fn recent_for_team(
        &self,
        team_id: i64,
        limit: usize,
    ) -> Result<Option<DailyTopicEdition>, DailyTopicsRepositoryError> {
        let Some(mut edition) = self.latest_for_team(team_id).await? else {
            return Ok(None);
        };
        edition.items = self.history_for_team(team_id, None, limit).await?;
        edition.items.truncate(limit);
        Ok(Some(edition))
    }

    async fn history_for_team(
        &self,
        team_id: i64,
        after: Option<&TopicPosition>,
        limit: usize,
    ) -> Result<Vec<DailyTopic>, DailyTopicsRepositoryError> {
        let rows = sqlx::query(r#"
            WITH topics AS (
                SELECT e.published_at AS edition_time, e.id AS edition_id, entry.topic, entry.position
                FROM f_i_daily_topic_editions AS e
                CROSS JOIN LATERAL jsonb_array_elements(
                    CASE WHEN jsonb_typeof(e.payload->'items') = 'array'
                         THEN e.payload->'items' ELSE '"invalid"'::jsonb END
                ) WITH ORDINALITY AS entry(topic, position)
                WHERE e.team_id = $1
            ), unique_topics AS (
                SELECT DISTINCT ON (topic->>'id') topic
                FROM topics ORDER BY topic->>'id', edition_time DESC, edition_id DESC, position ASC
            )
            SELECT topic FROM unique_topics
            WHERE $2::timestamptz IS NULL
               OR (topic->>'published_at')::timestamptz < $2
               OR ((topic->>'published_at')::timestamptz = $2 AND topic->>'id' > $3)
            ORDER BY (topic->>'published_at')::timestamptz DESC, topic->>'id' ASC
            LIMIT $4
        "#).bind(team_id)
            .bind(after.map(|position| position.published_at.with_timezone(&Utc)))
            .bind(after.map(|position| position.id.as_str()))
            .bind((limit + 1) as i64)
            .fetch_all(&self.pool).await.map_err(storage_error)?;
        rows.into_iter()
            .map(|row| {
                let value: serde_json::Value = row.try_get("topic").map_err(invalid_row)?;
                parse_topic(value)
            })
            .collect()
    }

    async fn topic_for_team(
        &self,
        team_id: i64,
        id: &str,
    ) -> Result<Option<DailyTopic>, DailyTopicsRepositoryError> {
        let row = sqlx::query(
            r#"
            SELECT entry.topic FROM f_i_daily_topic_editions AS e
            CROSS JOIN LATERAL jsonb_array_elements(
                CASE WHEN jsonb_typeof(e.payload->'items') = 'array'
                     THEN e.payload->'items' ELSE '"invalid"'::jsonb END
            ) WITH ORDINALITY AS entry(topic, position)
            WHERE e.team_id = $1 AND entry.topic->>'id' = $2
            ORDER BY e.published_at DESC, e.id DESC, entry.position ASC LIMIT 1
        "#,
        )
        .bind(team_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(storage_error)?;
        row.map(|row| {
            let value: serde_json::Value = row.try_get("topic").map_err(invalid_row)?;
            parse_topic(value)
        })
        .transpose()
    }
}

fn parse_topic(value: serde_json::Value) -> Result<DailyTopic, DailyTopicsRepositoryError> {
    serde_json::from_value::<StoredTopic>(value)
        .map(Into::into)
        .map_err(|error| {
            tracing::error!(%error, "invalid stored topic");
            DailyTopicsRepositoryError::InvalidEdition
        })
}

fn storage_error(error: sqlx::Error) -> DailyTopicsRepositoryError {
    tracing::error!(%error, "topic archive database query failed");
    DailyTopicsRepositoryError::Unavailable
}

fn invalid_row(error: sqlx::Error) -> DailyTopicsRepositoryError {
    tracing::error!(%error, "invalid daily topics database row");
    DailyTopicsRepositoryError::InvalidEdition
}
