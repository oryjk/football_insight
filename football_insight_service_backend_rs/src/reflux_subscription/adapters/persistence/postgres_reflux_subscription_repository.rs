use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::reflux_subscription::{
    domain::subscription::{
        NotificationTarget, RefluxEmailSubscriber, RefluxNotificationJob, RefluxSubscriptionPlan,
        RefluxSubscriptionScope, RefluxSubscriptionStatus, UserRefluxSubscription,
        normalize_team_code,
    },
    application::process_reflux_notification_jobs::should_retry_email_job,
    ports::reflux_subscription_repository::{
        CreateNotificationJobInput, CreateRefluxSubscriptionInput, RefluxSubscriptionRepository,
    },
};

pub struct PostgresRefluxSubscriptionRepository {
    pool: PgPool,
}

impl PostgresRefluxSubscriptionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RefluxSubscriptionRepository for PostgresRefluxSubscriptionRepository {
    async fn list_enabled_plans(&self) -> anyhow::Result<Vec<RefluxSubscriptionPlan>> {
        let rows = sqlx::query(
            r#"
            SELECT code, scope, team_code, season, title, description, price_cents,
                   enabled, sort_order, expires_at
              FROM f_i_reflux_subscription_plans
             WHERE enabled = TRUE
             ORDER BY sort_order ASC, code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|row| map_plan(&row)).collect()
    }

    async fn find_enabled_plan(
        &self,
        team_code: &str,
        plan_code: &str,
    ) -> anyhow::Result<Option<RefluxSubscriptionPlan>> {
        let normalized_team_code = normalize_team_code(team_code);
        let row = sqlx::query(
            r#"
            SELECT code, scope, team_code, season, title, description, price_cents,
                   enabled, sort_order, expires_at
              FROM f_i_reflux_subscription_plans
             WHERE code = $1
               AND enabled = TRUE
               AND team_code IN ($2, 'global')
             ORDER BY CASE WHEN team_code = $2 THEN 0 ELSE 1 END,
                      sort_order ASC
             LIMIT 1
            "#,
        )
        .bind(plan_code.trim())
        .bind(normalized_team_code)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| map_plan(&row)).transpose()
    }

    async fn list_user_active_subscriptions(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<UserRefluxSubscription>> {
        let rows = sqlx::query(
            r#"
            SELECT scope, team_code, season, match_id, status, starts_at, expires_at
              FROM f_i_user_reflux_subscriptions
             WHERE user_id = $1
               AND status = 'active'
               AND starts_at <= NOW()
               AND (expires_at IS NULL OR expires_at > NOW())
             ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|row| map_subscription(&row)).collect()
    }

    async fn get_user_email_target(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Option<NotificationTarget>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, channel, target, is_active
              FROM f_i_user_notification_targets
             WHERE user_id = $1
               AND channel = 'email'
               AND is_active = TRUE
             LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| map_notification_target(&row)))
    }

    async fn upsert_user_email_target(
        &self,
        user_id: Uuid,
        email: &str,
    ) -> anyhow::Result<NotificationTarget> {
        let row = sqlx::query(
            r#"
            INSERT INTO f_i_user_notification_targets (user_id, channel, target, is_active)
            VALUES ($1, 'email', $2, TRUE)
            ON CONFLICT (user_id, channel) DO UPDATE
               SET target = EXCLUDED.target,
                   is_active = TRUE,
                   updated_at = NOW()
            RETURNING id, user_id, channel, target, is_active
            "#,
        )
        .bind(user_id)
        .bind(email.trim())
        .fetch_one(&self.pool)
        .await?;

        Ok(map_notification_target(&row))
    }

    async fn create_subscription(
        &self,
        input: CreateRefluxSubscriptionInput,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO f_i_user_reflux_subscriptions (
                user_id, plan_code, scope, team_code, season, match_id,
                order_no, starts_at, expires_at, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'active')
            "#,
        )
        .bind(input.user_id)
        .bind(input.plan_code)
        .bind(input.scope)
        .bind(input.team_code)
        .bind(input.season)
        .bind(input.match_id)
        .bind(input.order_no)
        .bind(input.starts_at)
        .bind(input.expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn create_notification_job(
        &self,
        input: CreateNotificationJobInput,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO f_i_reflux_notification_jobs (
                user_id, target_id, team_code, match_id,
                subject, body_html, payload_json, status, next_attempt_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', NOW())
            "#,
        )
        .bind(input.user_id)
        .bind(input.target_id)
        .bind(input.team_code)
        .bind(input.match_id)
        .bind(input.subject)
        .bind(input.body_html)
        .bind(input.payload_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_cursor(
        &self,
        team_code: &str,
        match_id: i64,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        sqlx::query_scalar(
            r#"
            SELECT last_processed_at
              FROM f_i_reflux_notification_cursors
             WHERE team_code = $1
               AND match_id = $2
             LIMIT 1
            "#,
        )
        .bind(normalize_team_code(team_code))
        .bind(match_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn update_cursor(
        &self,
        team_code: &str,
        match_id: i64,
        last_processed_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO f_i_reflux_notification_cursors (team_code, match_id, last_processed_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (team_code, match_id) DO UPDATE
               SET last_processed_at = GREATEST(
                       f_i_reflux_notification_cursors.last_processed_at,
                       EXCLUDED.last_processed_at
                   ),
                   updated_at = NOW()
            "#,
        )
        .bind(normalize_team_code(team_code))
        .bind(match_id)
        .bind(last_processed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_email_subscribers_for_match(
        &self,
        team_code: &str,
        season: i32,
        match_id: i64,
    ) -> anyhow::Result<Vec<RefluxEmailSubscriber>> {
        let rows = sqlx::query(
            r#"
            SELECT subscriptions.user_id,
                   subscriptions.scope,
                   subscriptions.team_code,
                   subscriptions.season,
                   subscriptions.match_id,
                   subscriptions.status,
                   subscriptions.starts_at,
                   subscriptions.expires_at,
                   targets.id AS target_id,
                   targets.channel,
                   targets.target,
                   targets.is_active
              FROM f_i_user_reflux_subscriptions AS subscriptions
              JOIN f_i_user_notification_targets AS targets
                ON targets.user_id = subscriptions.user_id
               AND targets.channel = 'email'
               AND targets.is_active = TRUE
             WHERE subscriptions.team_code = $1
               AND subscriptions.status = 'active'
               AND subscriptions.starts_at <= NOW()
               AND (subscriptions.expires_at IS NULL OR subscriptions.expires_at > NOW())
               AND (
                    (subscriptions.scope = 'single_match' AND subscriptions.match_id = $2)
                 OR (subscriptions.scope = 'season' AND subscriptions.season = $3)
                 OR subscriptions.scope = 'lifetime'
               )
            "#,
        )
        .bind(normalize_team_code(team_code))
        .bind(match_id)
        .bind(season)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(RefluxEmailSubscriber {
                    user_id: row.get("user_id"),
                    target: NotificationTarget {
                        id: row.get("target_id"),
                        user_id: row.get("user_id"),
                        channel: row.get("channel"),
                        target: row.get("target"),
                        is_active: row.get("is_active"),
                    },
                    subscription: map_subscription(&row)?,
                })
            })
            .collect()
    }

    async fn list_pending_notification_jobs(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<RefluxNotificationJob>> {
        let rows = sqlx::query(
            r#"
            SELECT jobs.id,
                   jobs.user_id,
                   jobs.subject,
                   jobs.body_html,
                   jobs.attempts,
                   targets.id AS target_id,
                   targets.channel,
                   targets.target,
                   targets.is_active
              FROM f_i_reflux_notification_jobs AS jobs
              JOIN f_i_user_notification_targets AS targets
                ON targets.id = jobs.target_id
             WHERE jobs.status = 'pending'
               AND jobs.next_attempt_at <= NOW()
               AND targets.is_active = TRUE
             ORDER BY jobs.created_at ASC
             LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| RefluxNotificationJob {
                id: row.get("id"),
                target: NotificationTarget {
                    id: row.get("target_id"),
                    user_id: row.get("user_id"),
                    channel: row.get("channel"),
                    target: row.get("target"),
                    is_active: row.get("is_active"),
                },
                subject: row.get("subject"),
                body_html: row.get("body_html"),
                attempts: row.get("attempts"),
            })
            .collect())
    }

    async fn mark_notification_job_sent(&self, job_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE f_i_reflux_notification_jobs
               SET status = 'sent',
                   attempts = attempts + 1,
                   sent_at = NOW(),
                   updated_at = NOW()
             WHERE id = $1
            "#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn mark_notification_job_failed(
        &self,
        job_id: Uuid,
        attempts: i32,
        error: &str,
    ) -> anyhow::Result<()> {
        let status = if should_retry_email_job(attempts) {
            "pending"
        } else {
            "failed"
        };
        let next_attempt_at = Utc::now() + chrono::Duration::minutes(1);

        sqlx::query(
            r#"
            UPDATE f_i_reflux_notification_jobs
               SET status = $2,
                   attempts = $3,
                   next_attempt_at = $4,
                   last_error = $5,
                   updated_at = NOW()
             WHERE id = $1
            "#,
        )
        .bind(job_id)
        .bind(status)
        .bind(attempts)
        .bind(next_attempt_at)
        .bind(error)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn map_plan(row: &sqlx::postgres::PgRow) -> anyhow::Result<RefluxSubscriptionPlan> {
    let scope: String = row.get("scope");

    Ok(RefluxSubscriptionPlan {
        code: row.get("code"),
        scope: match scope.as_str() {
            "single_match" => RefluxSubscriptionScope::SingleMatch,
            "season" => RefluxSubscriptionScope::Season,
            "lifetime" => RefluxSubscriptionScope::Lifetime,
            _ => anyhow::bail!("unknown reflux subscription scope: {scope}"),
        },
        team_code: row.get("team_code"),
        season: row.get("season"),
        title: row.get("title"),
        description: row.get("description"),
        price_cents: row.get("price_cents"),
        enabled: row.get("enabled"),
        sort_order: row.get("sort_order"),
        expires_at: row.get::<Option<DateTime<Utc>>, _>("expires_at"),
    })
}

fn map_subscription(row: &sqlx::postgres::PgRow) -> anyhow::Result<UserRefluxSubscription> {
    let scope: String = row.get("scope");
    let status: String = row.get("status");

    Ok(UserRefluxSubscription {
        scope: match scope.as_str() {
            "single_match" => RefluxSubscriptionScope::SingleMatch,
            "season" => RefluxSubscriptionScope::Season,
            "lifetime" => RefluxSubscriptionScope::Lifetime,
            _ => anyhow::bail!("unknown reflux subscription scope: {scope}"),
        },
        team_code: row.get("team_code"),
        season: row.get("season"),
        match_id: row.get("match_id"),
        status: match status.as_str() {
            "active" => RefluxSubscriptionStatus::Active,
            "expired" => RefluxSubscriptionStatus::Expired,
            "cancelled" => RefluxSubscriptionStatus::Cancelled,
            _ => anyhow::bail!("unknown reflux subscription status: {status}"),
        },
        starts_at: row.get("starts_at"),
        expires_at: row.get("expires_at"),
    })
}

fn map_notification_target(row: &sqlx::postgres::PgRow) -> NotificationTarget {
    NotificationTarget {
        id: row.get("id"),
        user_id: row.get("user_id"),
        channel: row.get("channel"),
        target: row.get("target"),
        is_active: row.get("is_active"),
    }
}
