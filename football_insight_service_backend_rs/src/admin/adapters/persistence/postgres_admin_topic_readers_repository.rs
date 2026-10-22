use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::admin::{
    domain::admin_topic_readers::{AdminTopicReadersError, TopicReader, TopicReaderTopic},
    ports::admin_topic_readers_repository::AdminTopicReadersRepository,
};

pub struct PostgresAdminTopicReadersRepository {
    pool: PgPool,
}

impl PostgresAdminTopicReadersRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// A topic can appear in several editions; keep its most recent one.
pub const RECENT_TOPICS_SQL: &str = r#"
SELECT t.team_id, t.topic_id, t.title, t.edition_date,
       (SELECT COUNT(*) FROM f_i_daily_topic_readers r
         WHERE r.team_id = t.team_id AND r.topic_id = t.topic_id)::bigint AS reader_count
FROM (
    SELECT DISTINCT ON (e.team_id, entry.topic->>'id')
           e.team_id, entry.topic->>'id' AS topic_id,
           COALESCE(entry.topic->>'title', '') AS title,
           e.edition_date, e.published_at
    FROM f_i_daily_topic_editions e
    CROSS JOIN LATERAL jsonb_array_elements(
        CASE WHEN jsonb_typeof(e.payload->'items')='array' THEN e.payload->'items' ELSE '[]'::jsonb END
    ) entry(topic)
    WHERE entry.topic->>'id' IS NOT NULL
    ORDER BY e.team_id, entry.topic->>'id', e.published_at DESC, e.id DESC
) t
ORDER BY t.published_at DESC, t.topic_id
LIMIT $1
"#;

pub const READERS_SQL: &str = r#"
SELECT u.id AS user_id, u.display_name, u.avatar_url, r.first_read_at
FROM f_i_daily_topic_readers r
JOIN f_i_users u ON u.id = r.user_id
WHERE r.team_id = $1 AND r.topic_id = $2
ORDER BY r.first_read_at, u.id
"#;

fn map_error(error: sqlx::Error) -> AdminTopicReadersError {
    tracing::warn!(%error, "admin topic readers query failed");
    AdminTopicReadersError::Internal
}

#[async_trait]
impl AdminTopicReadersRepository for PostgresAdminTopicReadersRepository {
    async fn recent_topics(
        &self,
        limit: i64,
    ) -> Result<Vec<TopicReaderTopic>, AdminTopicReadersError> {
        let rows = sqlx::query(RECENT_TOPICS_SQL)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.iter()
            .map(|row| {
                Ok(TopicReaderTopic {
                    team_id: row.try_get("team_id")?,
                    topic_id: row.try_get("topic_id")?,
                    title: row.try_get("title")?,
                    edition_date: row.try_get("edition_date")?,
                    reader_count: row.try_get("reader_count")?,
                })
            })
            .collect::<Result<_, sqlx::Error>>()
            .map_err(map_error)
    }

    async fn readers(
        &self,
        team_id: i64,
        topic_id: &str,
    ) -> Result<Vec<TopicReader>, AdminTopicReadersError> {
        let rows = sqlx::query(READERS_SQL)
            .bind(team_id)
            .bind(topic_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.iter()
            .map(|row| {
                Ok(TopicReader {
                    user_id: row.try_get("user_id")?,
                    display_name: row.try_get("display_name")?,
                    avatar_url: row.try_get("avatar_url")?,
                    first_read_at: row.try_get("first_read_at")?,
                })
            })
            .collect::<Result<_, sqlx::Error>>()
            .map_err(map_error)
    }
}
