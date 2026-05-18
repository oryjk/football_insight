use std::sync::Arc;

use chrono::Utc;
use rand::Rng;
use uuid::Uuid;

use crate::{
    auth::ports::user_membership_port::UserMembershipPort,
    payment::{
        domain::order::{NewPaymentOrder, WxPayParams, reflux_subscription_product_type},
        ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
    },
    reflux_subscription::{
        domain::subscription::{
            RefluxSubscriptionPlan, RefluxSubscriptionScope, is_valid_notification_email,
            normalize_team_code,
        },
        ports::reflux_subscription_repository::RefluxSubscriptionRepository,
    },
};

pub struct CreateRefluxSubscriptionOrderUseCase {
    repository: Arc<dyn RefluxSubscriptionRepository>,
    order_repository: Arc<dyn OrderRepository>,
    user_membership_port: Arc<dyn UserMembershipPort>,
    wechat_pay_port: Arc<dyn WechatPayPort>,
}

impl CreateRefluxSubscriptionOrderUseCase {
    pub fn new(
        repository: Arc<dyn RefluxSubscriptionRepository>,
        order_repository: Arc<dyn OrderRepository>,
        user_membership_port: Arc<dyn UserMembershipPort>,
        wechat_pay_port: Arc<dyn WechatPayPort>,
    ) -> Self {
        Self {
            repository,
            order_repository,
            user_membership_port,
            wechat_pay_port,
        }
    }

    pub async fn execute(
        &self,
        input: CreateRefluxSubscriptionOrderInput,
    ) -> anyhow::Result<CreateRefluxSubscriptionOrderOutput> {
        let email = input.email.trim();
        if !is_valid_notification_email(email) {
            anyhow::bail!("请输入有效的邮箱地址");
        }

        let team_code = normalize_team_code(&input.team_code);
        let plan = self
            .repository
            .find_enabled_plan(&team_code, &input.plan_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("请选择有效的提醒套餐"))?;

        validate_match_binding(&plan, input.match_id)?;

        let openid = self
            .user_membership_port
            .get_user_open_id(input.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("请先绑定微信"))?;

        self.repository
            .upsert_user_email_target(input.user_id, email)
            .await?;

        let order_no = generate_order_no();
        self.order_repository
            .create_order(NewPaymentOrder {
                order_no: order_no.clone(),
                user_id: input.user_id,
                amount_cents: plan.price_cents,
                product_type: reflux_subscription_product_type(
                    &plan.code,
                    &team_code,
                    effective_product_match_id(&plan, input.match_id),
                ),
            })
            .await?;

        let wx_pay_params = self
            .wechat_pay_port
            .unified_order(&order_no, &plan.title, plan.price_cents, &openid)
            .await?;

        Ok(CreateRefluxSubscriptionOrderOutput {
            order_no,
            wx_pay_params,
        })
    }
}

fn validate_match_binding(plan: &RefluxSubscriptionPlan, match_id: Option<i64>) -> anyhow::Result<()> {
    if plan.scope == RefluxSubscriptionScope::SingleMatch && match_id.is_none() {
        anyhow::bail!("单场订阅需要选择比赛");
    }

    Ok(())
}

fn effective_product_match_id(plan: &RefluxSubscriptionPlan, match_id: Option<i64>) -> Option<i64> {
    match plan.scope {
        RefluxSubscriptionScope::SingleMatch => match_id,
        RefluxSubscriptionScope::Season | RefluxSubscriptionScope::Lifetime => None,
    }
}

fn generate_order_no() -> String {
    let timestamp = Utc::now().timestamp_millis();
    let random: u32 = rand::rng().random_range(1000..10000);
    format!("{}{}", timestamp, random)
}

pub struct CreateRefluxSubscriptionOrderInput {
    pub user_id: Uuid,
    pub plan_code: String,
    pub team_code: String,
    pub match_id: Option<i64>,
    pub email: String,
}

#[derive(Debug)]
pub struct CreateRefluxSubscriptionOrderOutput {
    pub order_no: String,
    pub wx_pay_params: WxPayParams,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{CreateRefluxSubscriptionOrderInput, CreateRefluxSubscriptionOrderUseCase};
    use crate::{
        auth::ports::user_membership_port::UserMembershipPort,
        payment::{
            domain::order::{NewPaymentOrder, OrderStatus, PaymentOrder, WxPayParams},
            ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
        },
        reflux_subscription::{
            domain::subscription::{
                NotificationTarget, RefluxSubscriptionPlan, RefluxSubscriptionScope,
                UserRefluxSubscription,
            },
            ports::reflux_subscription_repository::{
                CreateNotificationJobInput, CreateRefluxSubscriptionInput,
                RefluxSubscriptionRepository,
            },
        },
    };

    struct FakeRepository {
        plan: Option<RefluxSubscriptionPlan>,
        saved_emails: Mutex<Vec<String>>,
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
            unreachable!()
        }

        async fn upsert_user_email_target(
            &self,
            user_id: Uuid,
            email: &str,
        ) -> anyhow::Result<NotificationTarget> {
            self.saved_emails
                .lock()
                .expect("saved emails")
                .push(email.to_string());

            Ok(NotificationTarget {
                id: Uuid::parse_str("cccccccc-cccc-cccc-cccc-cccccccccccc").unwrap(),
                user_id,
                channel: "email".to_string(),
                target: email.to_string(),
                is_active: true,
            })
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

    struct FakeUserMembershipPort {
        open_id: Option<String>,
    }

    #[async_trait]
    impl UserMembershipPort for FakeUserMembershipPort {
        async fn get_user_open_id(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            Ok(self.open_id.clone())
        }

        async fn get_user_membership_tier(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            unreachable!()
        }

        async fn update_user_membership_tier(
            &self,
            _user_id: Uuid,
            _tier: &str,
        ) -> anyhow::Result<()> {
            unreachable!()
        }
    }

    #[derive(Default)]
    struct FakeOrderRepository {
        created_orders: Mutex<Vec<NewPaymentOrder>>,
    }

    #[async_trait]
    impl OrderRepository for FakeOrderRepository {
        async fn create_order(&self, order: NewPaymentOrder) -> anyhow::Result<PaymentOrder> {
            self.created_orders
                .lock()
                .expect("created orders")
                .push(order.clone());

            Ok(PaymentOrder {
                id: Uuid::new_v4(),
                order_no: order.order_no,
                user_id: order.user_id,
                amount_cents: order.amount_cents,
                status: OrderStatus::Pending,
                prepay_id: None,
                transaction_id: None,
                product_type: order.product_type,
                paid_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        async fn find_order_by_no(&self, _order_no: &str) -> anyhow::Result<Option<PaymentOrder>> {
            unreachable!()
        }
    }

    #[derive(Default)]
    struct FakeWechatPayPort {
        amounts: Mutex<Vec<i32>>,
    }

    #[async_trait]
    impl WechatPayPort for FakeWechatPayPort {
        async fn unified_order(
            &self,
            _order_no: &str,
            _description: &str,
            amount_cents: i32,
            _openid: &str,
        ) -> anyhow::Result<WxPayParams> {
            self.amounts
                .lock()
                .expect("wechat amounts")
                .push(amount_cents);
            Ok(WxPayParams {
                time_stamp: "1".to_string(),
                nonce_str: "nonce".to_string(),
                package: "prepay_id=test".to_string(),
                sign_type: "MD5".to_string(),
                pay_sign: "sign".to_string(),
            })
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
    async fn execute_creates_single_match_order_from_database_price_and_saves_email() {
        let repository = Arc::new(FakeRepository {
            plan: Some(plan(RefluxSubscriptionScope::SingleMatch)),
            saved_emails: Mutex::new(vec![]),
        });
        let order_repository = Arc::new(FakeOrderRepository::default());
        let wechat_pay_port = Arc::new(FakeWechatPayPort::default());
        let use_case = CreateRefluxSubscriptionOrderUseCase::new(
            repository.clone(),
            order_repository.clone(),
            Arc::new(FakeUserMembershipPort {
                open_id: Some("openid".to_string()),
            }),
            wechat_pay_port.clone(),
        );

        let output = use_case
            .execute(CreateRefluxSubscriptionOrderInput {
                user_id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
                plan_code: "single_match".to_string(),
                team_code: " ChengDu ".to_string(),
                match_id: Some(571),
                email: " user@example.com ".to_string(),
            })
            .await
            .expect("order");

        assert!(!output.order_no.is_empty());
        assert_eq!(
            repository.saved_emails.lock().expect("saved emails").as_slice(),
            ["user@example.com"]
        );
        let created_orders = order_repository
            .created_orders
            .lock()
            .expect("created orders");
        assert_eq!(created_orders.len(), 1);
        assert_eq!(created_orders[0].amount_cents, 500);
        assert_eq!(
            created_orders[0].product_type,
            "reflux_subscription:single_match:chengdu:571"
        );
        assert_eq!(
            wechat_pay_port.amounts.lock().expect("amounts").as_slice(),
            [500]
        );
    }

    #[tokio::test]
    async fn execute_rejects_single_match_order_without_match_id() {
        let repository = Arc::new(FakeRepository {
            plan: Some(plan(RefluxSubscriptionScope::SingleMatch)),
            saved_emails: Mutex::new(vec![]),
        });
        let use_case = CreateRefluxSubscriptionOrderUseCase::new(
            repository,
            Arc::new(FakeOrderRepository::default()),
            Arc::new(FakeUserMembershipPort {
                open_id: Some("openid".to_string()),
            }),
            Arc::new(FakeWechatPayPort::default()),
        );

        let error = use_case
            .execute(CreateRefluxSubscriptionOrderInput {
                user_id: Uuid::new_v4(),
                plan_code: "single_match".to_string(),
                team_code: "chengdu".to_string(),
                match_id: None,
                email: "user@example.com".to_string(),
            })
            .await
            .expect_err("single match requires match");

        assert!(error.to_string().contains("需要选择比赛"));
    }
}
