use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    admin::{
        domain::admin_telegraph_comments::{
            AdminTelegraphComment, AdminTelegraphCommentStatus, AdminTelegraphCommentStatusFilter,
            AdminTelegraphCommentsError, AdminTelegraphCommentsView,
        },
        ports::admin_telegraph_comments_repository::AdminTelegraphCommentsRepository,
    },
    auth::domain::user::sanitize_public_display_name,
};

const CONFIG_KEY: &str = "telegraph_comments_enabled";

pub struct PostgresAdminTelegraphCommentsRepository {
    pool: PgPool,
}

impl PostgresAdminTelegraphCommentsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn read_comments_enabled(&self) -> Result<bool, AdminTelegraphCommentsError> {
        let value = sqlx::query_scalar::<_, String>(
            "SELECT config_value FROM f_i_system_configs WHERE config_key = $1 LIMIT 1",
        )
        .bind(CONFIG_KEY)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;
        Ok(value
            .as_deref()
            .is_none_or(|raw| !raw.trim().eq_ignore_ascii_case("false")))
    }
}

fn is_undefined_table(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("42P01")
    )
}

fn map_error(error: sqlx::Error) -> AdminTelegraphCommentsError {
    tracing::warn!(%error, "admin telegraph comments query failed");
    AdminTelegraphCommentsError::Internal
}

#[async_trait]
impl AdminTelegraphCommentsRepository for PostgresAdminTelegraphCommentsRepository {
    async fn list(
        &self,
        filter: AdminTelegraphCommentStatusFilter,
        limit: usize,
    ) -> Result<AdminTelegraphCommentsView, AdminTelegraphCommentsError> {
        let comments_enabled = self.read_comments_enabled().await?;
        let status_clause = match filter {
            AdminTelegraphCommentStatusFilter::All => "c.status IN ('visible', 'hidden')",
            AdminTelegraphCommentStatusFilter::Visible => "c.status = 'visible'",
            AdminTelegraphCommentStatusFilter::Hidden => "c.status = 'hidden'",
        };
        let query = format!(
            r#"
            SELECT c.id,
                   c.team_id,
                   c.topic_id,
                   c.entry_id,
                   COALESCE(et.title, '') AS entry_title,
                   c.user_id,
                   u.display_name AS author_display_name,
                   u.avatar_url AS author_avatar_url,
                   c.content,
                   c.status,
                   c.created_at
              FROM f_i_telegraph_comments c
              JOIN f_i_users u ON u.id = c.user_id
              LEFT JOIN LATERAL (
                  SELECT entry.item->>'title' AS title
                    FROM f_i_daily_topic_editions e
                   CROSS JOIN LATERAL jsonb_array_elements(
                       CASE WHEN jsonb_typeof(e.payload->'items') = 'array'
                            THEN e.payload->'items' ELSE '[]'::jsonb END
                   ) AS topic_item(topic)
                   CROSS JOIN LATERAL jsonb_array_elements(
                       CASE WHEN jsonb_typeof(topic_item.topic->'entries') = 'array'
                            THEN topic_item.topic->'entries' ELSE '[]'::jsonb END
                   ) AS entry(item)
                   WHERE e.team_id = c.team_id
                     AND topic_item.topic->>'id' = c.topic_id
                     AND entry.item->>'id' = c.entry_id
                   ORDER BY e.published_at DESC, e.id DESC
                   LIMIT 1
              ) et ON TRUE
             WHERE {status_clause}
             ORDER BY c.created_at DESC, c.id DESC
             LIMIT $1
            "#
        );

        let rows = match sqlx::query(&query)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => rows,
            Err(error) if is_undefined_table(&error) => {
                tracing::warn!("f_i_telegraph_comments table missing; returning empty admin list");
                return Ok(AdminTelegraphCommentsView {
                    comments_enabled,
                    items: Vec::new(),
                });
            }
            Err(error) => return Err(map_error(error)),
        };

        let items = rows
            .into_iter()
            .map(|row| {
                let author_display_name: Option<String> = row.try_get("author_display_name")?;
                Ok(AdminTelegraphComment {
                    id: row.try_get("id")?,
                    team_id: row.try_get("team_id")?,
                    topic_id: row.try_get("topic_id")?,
                    entry_id: row.try_get("entry_id")?,
                    entry_title: row.try_get("entry_title")?,
                    user_id: row.try_get("user_id")?,
                    author_display_name: sanitize_public_display_name(
                        author_display_name.as_deref(),
                    ),
                    author_avatar_url: row.try_get("author_avatar_url")?,
                    content: row.try_get("content")?,
                    status: row.try_get("status")?,
                    created_at: row.try_get("created_at")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()
            .map_err(map_error)?;

        Ok(AdminTelegraphCommentsView {
            comments_enabled,
            items,
        })
    }

    async fn set_comments_enabled(
        &self,
        enabled: bool,
        admin_id: Uuid,
    ) -> Result<bool, AdminTelegraphCommentsError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let before = sqlx::query_scalar::<_, String>(
            "SELECT config_value FROM f_i_system_configs WHERE config_key = $1 LIMIT 1",
        )
        .bind(CONFIG_KEY)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_error)?;

        let value = if enabled { "true" } else { "false" };
        sqlx::query(
            r#"
            INSERT INTO f_i_system_configs (config_key, config_value, description)
            VALUES ($1, $2, '是否开启每日电报评论功能（true/false）')
            ON CONFLICT (config_key) DO UPDATE
               SET config_value = EXCLUDED.config_value,
                   updated_at = NOW()
            "#,
        )
        .bind(CONFIG_KEY)
        .bind(value)
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;

        sqlx::query(
            "INSERT INTO f_i_admin_audit_logs(id,admin_user_id,action,target_type,target_id,before_json,after_json) \
             VALUES($1,$2,'telegraph.comments.config.set','system_config',$3,$4,$5)",
        )
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .bind(CONFIG_KEY)
        .bind(serde_json::json!({ "comments_enabled": before }))
        .bind(serde_json::json!({ "comments_enabled": enabled }))
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;

        tx.commit().await.map_err(map_error)?;
        Ok(enabled)
    }

    async fn set_comment_status(
        &self,
        comment_id: Uuid,
        status: AdminTelegraphCommentStatus,
        admin_id: Uuid,
    ) -> Result<(), AdminTelegraphCommentsError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let before: Option<String> = sqlx::query_scalar(
            "SELECT status FROM f_i_telegraph_comments WHERE id = $1 LIMIT 1 FOR UPDATE",
        )
        .bind(comment_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_error)?;

        let Some(before) = before else {
            return Err(AdminTelegraphCommentsError::NotFound);
        };

        let next = status.as_str();
        sqlx::query(
            "UPDATE f_i_telegraph_comments SET status = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(comment_id)
        .bind(next)
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;

        sqlx::query(
            "INSERT INTO f_i_admin_audit_logs(id,admin_user_id,action,target_type,target_id,before_json,after_json) \
             VALUES($1,$2,'telegraph.comment.status.set','telegraph_comment',$3,$4,$5)",
        )
        .bind(Uuid::new_v4())
        .bind(admin_id)
        .bind(comment_id.to_string())
        .bind(serde_json::json!({ "status": before }))
        .bind(serde_json::json!({ "status": next }))
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;

        tx.commit().await.map_err(map_error)?;
        Ok(())
    }
}
