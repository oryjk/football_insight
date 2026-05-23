use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::reflux_subscription::domain::subscription::{
    NotificationTarget, RefluxEmailSubscriber, RefluxNotificationJob, RefluxSubscriptionPlan,
    UserRefluxSubscription,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRefluxSubscriptionInput {
    pub user_id: Uuid,
    pub plan_code: String,
    pub scope: String,
    pub team_code: String,
    pub season: Option<i32>,
    pub match_id: Option<i64>,
    pub order_no: String,
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct CreateNotificationJobInput {
    pub user_id: Uuid,
    pub target_id: Uuid,
    pub team_code: String,
    pub match_id: Option<i64>,
    pub subject: String,
    pub body_html: String,
    pub payload_json: serde_json::Value,
}

#[async_trait]
pub trait RefluxSubscriptionRepository: Send + Sync {
    async fn list_enabled_plans(&self) -> anyhow::Result<Vec<RefluxSubscriptionPlan>>;
    async fn find_enabled_plan(
        &self,
        team_code: &str,
        plan_code: &str,
    ) -> anyhow::Result<Option<RefluxSubscriptionPlan>>;
    async fn list_user_active_subscriptions(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<UserRefluxSubscription>>;
    async fn get_user_email_target(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Option<NotificationTarget>>;
    async fn upsert_user_email_target(
        &self,
        user_id: Uuid,
        email: &str,
    ) -> anyhow::Result<NotificationTarget>;
    async fn create_subscription(&self, input: CreateRefluxSubscriptionInput)
    -> anyhow::Result<()>;
    async fn create_notification_job(
        &self,
        input: CreateNotificationJobInput,
    ) -> anyhow::Result<()>;

    async fn get_cursor(
        &self,
        team_code: &str,
        match_id: i64,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let _ = (team_code, match_id);
        Ok(None)
    }

    async fn update_cursor(
        &self,
        team_code: &str,
        match_id: i64,
        last_processed_at: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let _ = (team_code, match_id, last_processed_at);
        Ok(())
    }

    async fn list_email_subscribers_for_match(
        &self,
        team_code: &str,
        season: i32,
        match_id: i64,
    ) -> anyhow::Result<Vec<RefluxEmailSubscriber>> {
        let _ = (team_code, season, match_id);
        Ok(vec![])
    }

    async fn list_pending_notification_jobs(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<RefluxNotificationJob>> {
        let _ = limit;
        Ok(vec![])
    }

    async fn mark_notification_job_sent(&self, job_id: Uuid) -> anyhow::Result<()> {
        let _ = job_id;
        Ok(())
    }

    async fn mark_notification_job_failed(
        &self,
        job_id: Uuid,
        attempts: i32,
        error: &str,
    ) -> anyhow::Result<()> {
        let _ = (job_id, attempts, error);
        Ok(())
    }
}
