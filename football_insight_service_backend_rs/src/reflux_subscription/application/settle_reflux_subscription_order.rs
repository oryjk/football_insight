use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::reflux_subscription::{
    domain::subscription::{RefluxSubscriptionPlan, RefluxSubscriptionScope, normalize_team_code},
    ports::reflux_subscription_repository::{
        CreateNotificationJobInput, CreateRefluxSubscriptionInput, RefluxSubscriptionRepository,
    },
};

pub struct SettleRefluxSubscriptionOrderUseCase {
    repository: Arc<dyn RefluxSubscriptionRepository>,
}

impl SettleRefluxSubscriptionOrderUseCase {
    pub fn new(repository: Arc<dyn RefluxSubscriptionRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: SettleRefluxSubscriptionOrderInput) -> anyhow::Result<()> {
        let team_code = normalize_team_code(&input.team_code);
        let plan = self
            .repository
            .find_enabled_plan(&team_code, &input.plan_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("提醒套餐已下架"))?;
        let target = self
            .repository
            .get_user_email_target(input.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("请先填写通知邮箱"))?;
        let now = Utc::now();
        let expires_at = resolve_subscription_expires_at(&plan, now);
        let subscription_match_id = match plan.scope {
            RefluxSubscriptionScope::SingleMatch => input.match_id,
            RefluxSubscriptionScope::Season | RefluxSubscriptionScope::Lifetime => None,
        };

        self.repository
            .create_subscription(CreateRefluxSubscriptionInput {
                user_id: input.user_id,
                plan_code: plan.code.clone(),
                scope: scope_as_str(&plan.scope).to_string(),
                team_code: team_code.clone(),
                season: plan.season,
                match_id: subscription_match_id,
                order_no: input.order_no.clone(),
                starts_at: now,
                expires_at,
            })
            .await?;

        self.repository
            .create_notification_job(CreateNotificationJobInput {
                user_id: input.user_id,
                target_id: target.id,
                team_code,
                match_id: subscription_match_id,
                subject: "[回流提醒] 订阅已开通".to_string(),
                body_html: build_welcome_email_html(&plan, expires_at),
                payload_json: serde_json::json!({
                    "kind": "reflux_subscription_welcome",
                    "plan_code": plan.code,
                    "order_no": input.order_no,
                }),
            })
            .await?;

        Ok(())
    }
}

fn resolve_subscription_expires_at(
    plan: &RefluxSubscriptionPlan,
    starts_at: chrono::DateTime<Utc>,
) -> Option<chrono::DateTime<Utc>> {
    match plan.scope {
        RefluxSubscriptionScope::SingleMatch => plan
            .expires_at
            .or(Some(starts_at + chrono::Duration::days(7))),
        RefluxSubscriptionScope::Season => plan.expires_at,
        RefluxSubscriptionScope::Lifetime => None,
    }
}

fn scope_as_str(scope: &RefluxSubscriptionScope) -> &'static str {
    match scope {
        RefluxSubscriptionScope::SingleMatch => "single_match",
        RefluxSubscriptionScope::Season => "season",
        RefluxSubscriptionScope::Lifetime => "lifetime",
    }
}

fn build_welcome_email_html(
    plan: &RefluxSubscriptionPlan,
    expires_at: Option<chrono::DateTime<Utc>>,
) -> String {
    let expiration = expires_at
        .map(|value| value.to_rfc3339())
        .unwrap_or_else(|| "长期有效".to_string());

    format!(
        "<p>你的「{}」已经开通。</p><p>有效期：{}</p><p>监控到新增回流后，我们会按分钟聚合发送邮件提醒。</p>",
        plan.title, expiration
    )
}

pub struct SettleRefluxSubscriptionOrderInput {
    pub order_no: String,
    pub user_id: Uuid,
    pub plan_code: String,
    pub team_code: String,
    pub match_id: Option<i64>,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{SettleRefluxSubscriptionOrderInput, SettleRefluxSubscriptionOrderUseCase};
    use crate::reflux_subscription::{
        domain::subscription::{
            NotificationTarget, RefluxSubscriptionPlan, RefluxSubscriptionScope,
            UserRefluxSubscription,
        },
        ports::reflux_subscription_repository::{
            CreateNotificationJobInput, CreateRefluxSubscriptionInput, RefluxSubscriptionRepository,
        },
    };

    struct FakeRepository {
        plan: Option<RefluxSubscriptionPlan>,
        target: Option<NotificationTarget>,
        subscriptions: Mutex<Vec<CreateRefluxSubscriptionInput>>,
        jobs: Mutex<Vec<CreateNotificationJobInput>>,
    }

    #[async_trait]
    impl RefluxSubscriptionRepository for FakeRepository {
        async fn list_enabled_plans(&self) -> anyhow::Result<Vec<RefluxSubscriptionPlan>> {
            unreachable!()
        }

        async fn find_enabled_plan(
            &self,
            _team_code: &str,
            _plan_code: &str,
        ) -> anyhow::Result<Option<RefluxSubscriptionPlan>> {
            Ok(self.plan.clone())
        }

        async fn list_user_active_subscriptions(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Vec<UserRefluxSubscription>> {
            unreachable!()
        }

        async fn get_user_email_target(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<NotificationTarget>> {
            Ok(self.target.clone())
        }

        async fn upsert_user_email_target(
            &self,
            _user_id: Uuid,
            _email: &str,
        ) -> anyhow::Result<NotificationTarget> {
            unreachable!()
        }

        async fn create_subscription(
            &self,
            input: CreateRefluxSubscriptionInput,
        ) -> anyhow::Result<()> {
            self.subscriptions
                .lock()
                .expect("subscriptions")
                .push(input);
            Ok(())
        }

        async fn create_notification_job(
            &self,
            input: CreateNotificationJobInput,
        ) -> anyhow::Result<()> {
            self.jobs.lock().expect("jobs").push(input);
            Ok(())
        }
    }

    fn target(user_id: Uuid) -> NotificationTarget {
        NotificationTarget {
            id: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap(),
            user_id,
            channel: "email".to_string(),
            target: "user@example.com".to_string(),
            is_active: true,
        }
    }

    fn plan(scope: RefluxSubscriptionScope) -> RefluxSubscriptionPlan {
        RefluxSubscriptionPlan {
            code: match scope {
                RefluxSubscriptionScope::SingleMatch => "single_match",
                RefluxSubscriptionScope::Season => "season_2026",
                RefluxSubscriptionScope::Lifetime => "lifetime",
            }
            .to_string(),
            scope,
            team_code: "global".to_string(),
            season: Some(2026),
            title: "回流提醒".to_string(),
            description: String::new(),
            price_cents: 500,
            enabled: true,
            sort_order: 10,
            expires_at: Some(Utc.with_ymd_and_hms(2026, 12, 31, 15, 59, 59).unwrap()),
        }
    }

    #[tokio::test]
    async fn execute_creates_single_match_subscription_and_welcome_job() {
        let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let repository = Arc::new(FakeRepository {
            plan: Some(plan(RefluxSubscriptionScope::SingleMatch)),
            target: Some(target(user_id)),
            subscriptions: Mutex::new(vec![]),
            jobs: Mutex::new(vec![]),
        });
        let use_case = SettleRefluxSubscriptionOrderUseCase::new(repository.clone());

        use_case
            .execute(SettleRefluxSubscriptionOrderInput {
                order_no: "202605180001".to_string(),
                user_id,
                plan_code: "single_match".to_string(),
                team_code: " ChengDu ".to_string(),
                match_id: Some(571),
            })
            .await
            .expect("settlement");

        let subscriptions = repository.subscriptions.lock().expect("subscriptions");
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].scope, "single_match");
        assert_eq!(subscriptions[0].team_code, "chengdu");
        assert_eq!(subscriptions[0].match_id, Some(571));
        assert_eq!(subscriptions[0].order_no, "202605180001");

        let jobs = repository.jobs.lock().expect("jobs");
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].subject, "[回流提醒] 订阅已开通");
        assert!(jobs[0].body_html.contains("已经开通"));
        assert_eq!(jobs[0].match_id, Some(571));
    }

    #[tokio::test]
    async fn execute_creates_season_subscription_without_match_binding() {
        let user_id = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let repository = Arc::new(FakeRepository {
            plan: Some(plan(RefluxSubscriptionScope::Season)),
            target: Some(target(user_id)),
            subscriptions: Mutex::new(vec![]),
            jobs: Mutex::new(vec![]),
        });
        let use_case = SettleRefluxSubscriptionOrderUseCase::new(repository.clone());

        use_case
            .execute(SettleRefluxSubscriptionOrderInput {
                order_no: "202605180002".to_string(),
                user_id,
                plan_code: "season_2026".to_string(),
                team_code: "chengdu".to_string(),
                match_id: Some(571),
            })
            .await
            .expect("settlement");

        let subscriptions = repository.subscriptions.lock().expect("subscriptions");
        assert_eq!(subscriptions[0].scope, "season");
        assert_eq!(subscriptions[0].match_id, None);
        assert_eq!(
            subscriptions[0].expires_at,
            Some(Utc.with_ymd_and_hms(2026, 12, 31, 15, 59, 59).unwrap())
        );
    }
}
