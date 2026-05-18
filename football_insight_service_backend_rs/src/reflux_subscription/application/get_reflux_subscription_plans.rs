use std::sync::Arc;

use uuid::Uuid;

use crate::reflux_subscription::{
    domain::subscription::{
        NotificationTarget, RefluxSubscriptionPlan, UserRefluxSubscription, select_effective_plans,
    },
    ports::reflux_subscription_repository::RefluxSubscriptionRepository,
};

pub struct GetRefluxSubscriptionPlansUseCase {
    repository: Arc<dyn RefluxSubscriptionRepository>,
}

impl GetRefluxSubscriptionPlansUseCase {
    pub fn new(repository: Arc<dyn RefluxSubscriptionRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        team_code: &str,
    ) -> anyhow::Result<RefluxSubscriptionPlansView> {
        let plans = self.repository.list_enabled_plans().await?;
        let subscriptions = self
            .repository
            .list_user_active_subscriptions(user_id)
            .await?;
        let email_target = self.repository.get_user_email_target(user_id).await?;

        Ok(RefluxSubscriptionPlansView {
            plans: select_effective_plans(plans, team_code),
            active_subscriptions: subscriptions,
            email_target,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RefluxSubscriptionPlansView {
    pub plans: Vec<RefluxSubscriptionPlan>,
    pub active_subscriptions: Vec<UserRefluxSubscription>,
    pub email_target: Option<NotificationTarget>,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::GetRefluxSubscriptionPlansUseCase;
    use crate::reflux_subscription::{
        domain::subscription::{NotificationTarget, RefluxSubscriptionPlan, UserRefluxSubscription},
        ports::reflux_subscription_repository::{
            CreateNotificationJobInput, CreateRefluxSubscriptionInput,
            RefluxSubscriptionRepository,
        },
    };

    struct FakeRepository {
        plans: Mutex<Vec<RefluxSubscriptionPlan>>,
    }

    #[async_trait]
    impl RefluxSubscriptionRepository for FakeRepository {
        async fn list_enabled_plans(&self) -> anyhow::Result<Vec<RefluxSubscriptionPlan>> {
            Ok(self.plans.lock().unwrap().clone())
        }

        async fn find_enabled_plan(
            &self,
            _team_code: &str,
            _plan_code: &str,
        ) -> anyhow::Result<Option<RefluxSubscriptionPlan>> {
            unreachable!()
        }

        async fn list_user_active_subscriptions(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Vec<UserRefluxSubscription>> {
            Ok(vec![])
        }

        async fn get_user_email_target(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<NotificationTarget>> {
            Ok(None)
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
            _input: CreateRefluxSubscriptionInput,
        ) -> anyhow::Result<()> {
            unreachable!()
        }

        async fn create_notification_job(
            &self,
            _input: CreateNotificationJobInput,
        ) -> anyhow::Result<()> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn execute_prefers_team_specific_plan_over_global_plan() {
        let repository = Arc::new(FakeRepository {
            plans: Mutex::new(vec![
                crate::reflux_subscription::domain::subscription::tests_support::test_plan(
                    "single_match",
                    "global",
                    500,
                    10,
                ),
                crate::reflux_subscription::domain::subscription::tests_support::test_plan(
                    "single_match",
                    "chengdu",
                    600,
                    10,
                ),
            ]),
        });
        let use_case = GetRefluxSubscriptionPlansUseCase::new(repository);

        let view = use_case
            .execute(Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(), "chengdu")
            .await
            .expect("plans");

        assert_eq!(view.plans.len(), 1);
        assert_eq!(view.plans[0].team_code, "chengdu");
        assert_eq!(view.plans[0].price_cents, 600);
    }
}

