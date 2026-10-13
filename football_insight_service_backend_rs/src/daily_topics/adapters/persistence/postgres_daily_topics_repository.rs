use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use sqlx::{PgPool, Row};

use crate::daily_topics::{
    domain::{
        archive::TopicPosition,
        edition::{DailyTopic, DailyTopicEdition},
        heat::{TopicHeat, TopicVisit},
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
    async fn heat_for_topics(
        &self,
        team_id: i64,
        ids: &[String],
    ) -> Result<std::collections::HashMap<String, TopicHeat>, DailyTopicsRepositoryError> {
        let rows = sqlx::query("SELECT topic_id, base_heat, view_count FROM f_i_daily_topic_metrics WHERE team_id=$1 AND topic_id=ANY($2)")
            .bind(team_id).bind(ids).fetch_all(&self.pool).await.map_err(storage_error)?;
        rows.into_iter()
            .map(|row| {
                Ok((
                    row.try_get("topic_id").map_err(invalid_row)?,
                    row_heat(&row)?,
                ))
            })
            .collect()
    }

    async fn record_visit(
        &self,
        visit: &TopicVisit,
    ) -> Result<Option<TopicHeat>, DailyTopicsRepositoryError> {
        let team_id = visit.topic.team_id();
        let id = visit.topic.id();
        let mut tx = self.pool.begin().await.map_err(storage_error)?;
        let exists: bool = sqlx::query_scalar(r#"
            SELECT EXISTS (
                SELECT 1 FROM f_i_daily_topic_editions e
                CROSS JOIN LATERAL jsonb_array_elements(
                    CASE WHEN jsonb_typeof(e.payload->'items')='array' THEN e.payload->'items' ELSE '[]'::jsonb END
                ) entry(topic)
                WHERE e.team_id=$1 AND entry.topic->>'id'=$2
            )
        "#).bind(team_id).bind(id).fetch_one(&mut *tx).await.map_err(storage_error)?;
        if !exists {
            return Ok(None);
        }
        sqlx::query("INSERT INTO f_i_daily_topic_metrics(team_id,topic_id) VALUES($1,$2) ON CONFLICT DO NOTHING")
            .bind(team_id).bind(id).execute(&mut *tx).await.map_err(storage_error)?;
        // Serialize visits for this topic before deduplication; simultaneous requests count exactly once.
        sqlx::query(
            "SELECT 1 FROM f_i_daily_topic_metrics WHERE team_id=$1 AND topic_id=$2 FOR UPDATE",
        )
        .bind(team_id)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(storage_error)?;
        let counted = sqlx::query(r#"
            INSERT INTO f_i_daily_topic_visitors(team_id,topic_id,visitor_hash) VALUES($1,$2,$3)
            ON CONFLICT(team_id,topic_id,visitor_hash) DO UPDATE SET last_counted_at=clock_timestamp()
            WHERE f_i_daily_topic_visitors.last_counted_at <= clock_timestamp() - INTERVAL '30 minutes'
            RETURNING 1
        "#).bind(team_id).bind(id).bind(&visit.visitor_hash).fetch_optional(&mut *tx).await.map_err(storage_error)?.is_some();
        if counted {
            sqlx::query("UPDATE f_i_daily_topic_metrics SET view_count=LEAST(view_count+1,9007199254740991),updated_at=clock_timestamp() WHERE team_id=$1 AND topic_id=$2")
                .bind(team_id).bind(id).execute(&mut *tx).await.map_err(storage_error)?;
        }
        let row = sqlx::query("SELECT base_heat,view_count FROM f_i_daily_topic_metrics WHERE team_id=$1 AND topic_id=$2")
            .bind(team_id).bind(id).fetch_one(&mut *tx).await.map_err(storage_error)?;
        let heat = row_heat(&row)?;
        tx.commit().await.map_err(storage_error)?;
        Ok(Some(heat))
    }

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

fn row_heat(row: &sqlx::postgres::PgRow) -> Result<TopicHeat, DailyTopicsRepositoryError> {
    let base: i64 = row.try_get("base_heat").map_err(invalid_row)?;
    let views: i64 = row.try_get("view_count").map_err(invalid_row)?;
    Ok(TopicHeat {
        base: base
            .try_into()
            .map_err(|_| DailyTopicsRepositoryError::InvalidEdition)?,
        views: views
            .try_into()
            .map_err(|_| DailyTopicsRepositoryError::InvalidEdition)?,
    })
}
