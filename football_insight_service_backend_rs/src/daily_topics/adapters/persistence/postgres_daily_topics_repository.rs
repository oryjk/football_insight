use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::daily_topics::{
    domain::{
        archive::TopicPosition,
        edition::{DailyTopic, DailyTopicEdition},
        heat::{TopicHeat, TopicVisit},
        interactions::EntryInteractionStats,
    },
    ports::daily_topics_repository::{DailyTopicsRepository, DailyTopicsRepositoryError},
};

use super::stored_payload::{StoredPayload, StoredTopic};

pub struct PostgresDailyTopicsRepository {
    pub(super) pool: PgPool,
}

impl PostgresDailyTopicsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub(super) async fn is_comments_enabled_internal(&self) -> Result<bool, sqlx::Error> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT config_value FROM f_i_system_configs WHERE config_key = 'telegraph_comments_enabled' LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await;
        match row {
            Ok(Some(value)) => Ok(!value.trim().eq_ignore_ascii_case("false")),
            Ok(None) => Ok(true),
            Err(error) if is_undefined_table(&error) => Ok(true),
            Err(error) => Err(error),
        }
    }
}

#[async_trait]
impl DailyTopicsRepository for PostgresDailyTopicsRepository {
    async fn timeline_for_tag(
        &self,
        team_id: i64,
        tag_team_id: i64,
        before: &TopicPosition,
        after: Option<&TopicPosition>,
        limit: usize,
    ) -> Result<Vec<DailyTopic>, DailyTopicsRepositoryError> {
        let rows = sqlx::query(r#"
            WITH topics AS (
                SELECT e.published_at AS edition_time, e.id AS edition_id, entry.topic, entry.position
                FROM f_i_daily_topic_editions e
                CROSS JOIN LATERAL jsonb_array_elements(e.payload->'items') WITH ORDINALITY entry(topic, position)
                WHERE e.team_id = $1
            ), unique_topics AS (
                SELECT DISTINCT ON (topic->>'id') topic FROM topics
                ORDER BY topic->>'id', edition_time DESC, edition_id DESC, position ASC
            )
            SELECT topic FROM unique_topics
            WHERE EXISTS (
                SELECT 1 FROM jsonb_array_elements(CASE WHEN jsonb_typeof(topic->'team_tags')='array'
                    THEN topic->'team_tags' ELSE '[]'::jsonb END) tag
                WHERE tag->'team_id' = to_jsonb($2::bigint)
            )
            AND (topic->>'published_at')::timestamptz < $3
            AND ($4::timestamptz IS NULL OR (topic->>'published_at')::timestamptz < $4
                OR ((topic->>'published_at')::timestamptz = $4 AND topic->>'id' > $5))
            ORDER BY (topic->>'published_at')::timestamptz DESC, topic->>'id' ASC LIMIT $6
        "#).bind(team_id).bind(tag_team_id).bind(before.published_at.with_timezone(&Utc))
            .bind(after.map(|p| p.published_at.with_timezone(&Utc))).bind(after.map(|p| p.id.as_str()))
            .bind((limit + 1) as i64).fetch_all(&self.pool).await.map_err(storage_error)?;
        rows.into_iter()
            .map(|row| parse_topic(row.try_get("topic").map_err(invalid_row)?))
            .collect()
    }

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
        if let Some(user_id) = visit.user_id {
            sqlx::query("INSERT INTO f_i_daily_topic_readers(team_id,topic_id,user_id) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
                .bind(team_id).bind(id).bind(user_id).execute(&mut *tx).await.map_err(storage_error)?;
        }
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

    async fn edition_for_date(
        &self,
        team_id: i64,
        edition_date: NaiveDate,
    ) -> Result<Option<DailyTopicEdition>, DailyTopicsRepositoryError> {
        let row = sqlx::query(
            "SELECT team_id, edition_date, published_at, payload FROM f_i_daily_topic_editions WHERE team_id = $1 AND edition_date = $2 ORDER BY (generator_version LIKE 'daily-roundup-%') DESC, published_at DESC, id DESC LIMIT 1"
        )
        .bind(team_id)
        .bind(edition_date)
        .fetch_optional(&self.pool)
        .await
        .map_err(|error| {
            tracing::error!(%error, team_id, %edition_date, "daily topics by-date query failed");
            DailyTopicsRepositoryError::Unavailable
        })?;
        let Some(row) = row else {
            return Ok(None);
        };
        let team_id: i64 = row.try_get("team_id").map_err(invalid_row)?;
        let edition_date: NaiveDate = row.try_get("edition_date").map_err(invalid_row)?;
        let published_at: DateTime<Utc> = row.try_get("published_at").map_err(invalid_row)?;
        let payload: serde_json::Value = row.try_get("payload").map_err(invalid_row)?;
        let payload: StoredPayload = serde_json::from_value(payload).map_err(|error| {
            tracing::error!(%error, team_id, %edition_date, "invalid daily topics payload");
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

    async fn comments_enabled(&self) -> Result<bool, DailyTopicsRepositoryError> {
        self.is_comments_enabled_internal()
            .await
            .map_err(storage_error)
    }

    async fn entry_interactions(
        &self,
        team_id: i64,
        topic_ids: &[String],
        viewer_user_id: Option<Uuid>,
    ) -> Result<std::collections::HashMap<String, EntryInteractionStats>, DailyTopicsRepositoryError>
    {
        let mut map = std::collections::HashMap::<String, EntryInteractionStats>::new();
        if topic_ids.is_empty() {
            return Ok(map);
        }

        let reaction_rows = sqlx::query(
            r#"
            SELECT r.entry_id,
                   COUNT(*) FILTER (WHERE r.reaction = 'like' AND u.status = 'active')::bigint AS likes_count,
                   COUNT(*) FILTER (WHERE r.reaction = 'dislike' AND u.status = 'active')::bigint AS dislikes_count,
                   COALESCE(BOOL_OR(r.user_id = $3 AND r.reaction = 'like' AND u.status = 'active'), FALSE) AS liked_by_me,
                   COALESCE(BOOL_OR(r.user_id = $3 AND r.reaction = 'dislike' AND u.status = 'active'), FALSE) AS disliked_by_me
              FROM f_i_telegraph_reactions r
              JOIN f_i_users u ON u.id = r.user_id
             WHERE r.team_id = $1
               AND r.topic_id = ANY($2)
             GROUP BY r.entry_id
            "#,
        )
        .bind(team_id)
        .bind(topic_ids)
        .bind(viewer_user_id)
        .fetch_all(&self.pool)
        .await;

        match reaction_rows {
            Ok(rows) => {
                for row in rows {
                    let entry_id: String = row.try_get("entry_id").map_err(invalid_row)?;
                    let likes_count: i64 = row.try_get("likes_count").map_err(invalid_row)?;
                    let dislikes_count: i64 = row.try_get("dislikes_count").map_err(invalid_row)?;
                    let liked_by_me: bool = row.try_get("liked_by_me").map_err(invalid_row)?;
                    let disliked_by_me: bool = row.try_get("disliked_by_me").map_err(invalid_row)?;
                    let stats = map.entry(entry_id).or_default();
                    stats.likes_count = u64::try_from(likes_count).unwrap_or(0);
                    stats.dislikes_count = u64::try_from(dislikes_count).unwrap_or(0);
                    stats.liked_by_me = liked_by_me;
                    stats.disliked_by_me = disliked_by_me;
                }
            }
            Err(error) if is_undefined_table(&error) => {
                tracing::warn!("f_i_telegraph_reactions table missing; returning default reaction counts");
                return Ok(map);
            }
            Err(error) => return Err(storage_error(error)),
        }

        let comment_rows = sqlx::query(
            r#"
            SELECT c.entry_id, COUNT(*)::bigint AS comments_count
              FROM f_i_telegraph_comments c
              JOIN f_i_users u ON u.id = c.user_id
             WHERE c.team_id = $1
               AND c.topic_id = ANY($2)
               AND c.status = 'visible'
               AND u.status = 'active'
             GROUP BY c.entry_id
            "#,
        )
        .bind(team_id)
        .bind(topic_ids)
        .fetch_all(&self.pool)
        .await;

        match comment_rows {
            Ok(rows) => {
                for row in rows {
                    let entry_id: String = row.try_get("entry_id").map_err(invalid_row)?;
                    let comments_count: i64 = row.try_get("comments_count").map_err(invalid_row)?;
                    map.entry(entry_id).or_default().comments_count =
                        u64::try_from(comments_count).unwrap_or(0);
                }
            }
            Err(error) if is_undefined_table(&error) => {
                tracing::warn!("f_i_telegraph_comments table missing; returning default comment counts");
            }
            Err(error) => return Err(storage_error(error)),
        }

        Ok(map)
    }
}

pub(super) fn is_undefined_table(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("42P01")
    )
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
