use std::sync::Arc;

use chrono::Utc;
use rand::Rng;
use uuid::Uuid;

use crate::{
    auth::{
        domain::membership::membership_tier_rank, ports::user_membership_port::UserMembershipPort,
    },
    match_id_unlock::{
        domain::match_id_unlock::{
            MATCH_ID_UNLOCK_MINIMUM_TIER, MATCH_ID_UNLOCK_ORDER_DESCRIPTION,
            MATCH_ID_UNLOCK_PRICE_CENTS, MatchIdUnlockError,
        },
        ports::{
            match_id_source::MatchIdSource, match_id_unlock_repository::MatchIdUnlockRepository,
        },
    },
    payment::{
        domain::order::{NewPaymentOrder, WxPayParams, match_id_unlock_product_type},
        ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
    },
};

pub struct CreateMatchIdOrderUseCase {
    repository: Arc<dyn MatchIdUnlockRepository>,
    match_id_source: Arc<dyn MatchIdSource>,
    order_repository: Arc<dyn OrderRepository>,
    user_membership_port: Arc<dyn UserMembershipPort>,
    wechat_pay_port: Arc<dyn WechatPayPort>,
}

impl CreateMatchIdOrderUseCase {
    pub fn new(
        repository: Arc<dyn MatchIdUnlockRepository>,
        match_id_source: Arc<dyn MatchIdSource>,
        order_repository: Arc<dyn OrderRepository>,
        user_membership_port: Arc<dyn UserMembershipPort>,
        wechat_pay_port: Arc<dyn WechatPayPort>,
    ) -> Self {
        Self {
            repository,
            match_id_source,
            order_repository,
            user_membership_port,
            wechat_pay_port,
        }
    }

    pub async fn execute(
        &self,
        input: CreateMatchIdOrderInput,
    ) -> anyhow::Result<CreateMatchIdOrderOutput> {
        if !self.match_id_source.known_match_id(input.match_id).await? {
            return Err(MatchIdUnlockError::MatchNotFound.into());
        }

        let effective_tier = self
            .user_membership_port
            .get_user_membership_tier(input.user_id)
            .await?
            .unwrap_or_else(|| "V1".to_string());
        if membership_tier_rank(&effective_tier)
            >= membership_tier_rank(MATCH_ID_UNLOCK_MINIMUM_TIER)
        {
            return Err(MatchIdUnlockError::MembershipTierSufficient.into());
        }

        if self
            .repository
            .find_unlock(input.user_id, input.match_id)
            .await?
        {
            return Err(MatchIdUnlockError::AlreadyUnlocked.into());
        }

        let openid = self
            .user_membership_port
            .get_user_open_id(input.user_id)
            .await?
            .ok_or(MatchIdUnlockError::WechatBindingRequired)?;

        let order_no = generate_order_no();
        self.order_repository
            .create_order(NewPaymentOrder {
                order_no: order_no.clone(),
                user_id: input.user_id,
                amount_cents: MATCH_ID_UNLOCK_PRICE_CENTS,
                product_type: match_id_unlock_product_type(input.match_id),
            })
            .await?;

        let wx_pay_params = self
            .wechat_pay_port
            .unified_order(
                &order_no,
                MATCH_ID_UNLOCK_ORDER_DESCRIPTION,
                MATCH_ID_UNLOCK_PRICE_CENTS,
                &openid,
            )
            .await?;

        Ok(CreateMatchIdOrderOutput {
            order_no,
            wx_pay_params,
        })
    }
}

fn generate_order_no() -> String {
    let timestamp = Utc::now().timestamp_millis();
    let random: u32 = rand::rng().random_range(1000..10000);
    format!("{timestamp}{random}")
}

pub struct CreateMatchIdOrderInput {
    pub user_id: Uuid,
    pub match_id: i64,
}

#[derive(Debug)]
pub struct CreateMatchIdOrderOutput {
    pub order_no: String,
    pub wx_pay_params: WxPayParams,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    use super::{CreateMatchIdOrderInput, CreateMatchIdOrderUseCase};
    use crate::{
        auth::ports::user_membership_port::UserMembershipPort,
        match_id_unlock::{
            domain::match_id_unlock::MatchIdUnlockError,
            ports::{
                match_id_source::MatchIdSource, match_id_unlock_repository::MatchIdUnlockRepository,
            },
        },
        payment::{
            domain::order::{NewPaymentOrder, OrderStatus, PaymentOrder, WxPayParams},
            ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
        },
    };

    struct FakeMatchIdSource {
        known: bool,
    }

    #[async_trait]
    impl MatchIdSource for FakeMatchIdSource {
        async fn known_match_id(&self, _match_id: i64) -> anyhow::Result<bool> {
            Ok(self.known)
        }
    }

    struct FakeRepository {
        unlocked_matches: Vec<i64>,
    }

    #[async_trait]
    impl MatchIdUnlockRepository for FakeRepository {
        async fn find_unlock(&self, _user_id: Uuid, match_id: i64) -> anyhow::Result<bool> {
            Ok(self.unlocked_matches.contains(&match_id))
        }
    }

    struct FakeUserMembershipPort {
        tier: Option<String>,
        open_id: Option<String>,
    }

    #[async_trait]
    impl UserMembershipPort for FakeUserMembershipPort {
        async fn get_user_open_id(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            Ok(self.open_id.clone())
        }

        async fn get_user_membership_tier(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            Ok(self.tier.clone())
        }

        async fn update_user_membership_tier(
            &self,
            _user_id: Uuid,
            _tier: &str,
        ) -> anyhow::Result<()> {
            unreachable!()
        }

        async fn is_seat_swap_notice_enabled(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<bool>> {
            Ok(Some(false))
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

    fn use_case(
        repository: FakeRepository,
        known_match: bool,
        membership: FakeUserMembershipPort,
    ) -> (
        CreateMatchIdOrderUseCase,
        Arc<FakeOrderRepository>,
        Arc<FakeWechatPayPort>,
    ) {
        let order_repository = Arc::new(FakeOrderRepository::default());
        let wechat_pay_port = Arc::new(FakeWechatPayPort::default());
        let use_case = CreateMatchIdOrderUseCase::new(
            Arc::new(repository),
            Arc::new(FakeMatchIdSource { known: known_match }),
            order_repository.clone(),
            Arc::new(membership),
            wechat_pay_port.clone(),
        );

        (use_case, order_repository, wechat_pay_port)
    }

    fn input() -> CreateMatchIdOrderInput {
        CreateMatchIdOrderInput {
            user_id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            match_id: 571,
        }
    }

    fn downcast_error(error: &anyhow::Error) -> &MatchIdUnlockError {
        error
            .downcast_ref::<MatchIdUnlockError>()
            .expect("typed match id unlock error")
    }

    #[tokio::test]
    async fn execute_creates_order_with_fixed_price_and_product_type() {
        let (use_case, order_repository, wechat_pay_port) = use_case(
            FakeRepository {
                unlocked_matches: vec![],
            },
            true,
            FakeUserMembershipPort {
                tier: Some("V5".to_string()),
                open_id: Some("openid".to_string()),
            },
        );

        let output = use_case.execute(input()).await.expect("order");

        assert!(!output.order_no.is_empty());
        let created_orders = order_repository
            .created_orders
            .lock()
            .expect("created orders");
        assert_eq!(created_orders.len(), 1);
        assert_eq!(created_orders[0].amount_cents, 500);
        assert_eq!(created_orders[0].product_type, "match_id_unlock:571");
        assert_eq!(
            wechat_pay_port.amounts.lock().expect("amounts").as_slice(),
            [500]
        );
    }

    #[tokio::test]
    async fn execute_rejects_membership_tier_at_v6() {
        let (use_case, order_repository, _) = use_case(
            FakeRepository {
                unlocked_matches: vec![],
            },
            true,
            FakeUserMembershipPort {
                tier: Some("V6".to_string()),
                open_id: Some("openid".to_string()),
            },
        );

        let error = use_case
            .execute(input())
            .await
            .expect_err("v6 members should not pay");

        assert!(matches!(
            downcast_error(&error),
            MatchIdUnlockError::MembershipTierSufficient
        ));
        assert_eq!(
            order_repository
                .created_orders
                .lock()
                .expect("created orders")
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn execute_rejects_already_unlocked_match() {
        let (use_case, _, _) = use_case(
            FakeRepository {
                unlocked_matches: vec![571],
            },
            true,
            FakeUserMembershipPort {
                tier: Some("V5".to_string()),
                open_id: Some("openid".to_string()),
            },
        );

        let error = use_case
            .execute(input())
            .await
            .expect_err("already unlocked match");

        assert!(matches!(
            downcast_error(&error),
            MatchIdUnlockError::AlreadyUnlocked
        ));
    }

    #[tokio::test]
    async fn execute_rejects_user_without_wechat_binding() {
        let (use_case, order_repository, _) = use_case(
            FakeRepository {
                unlocked_matches: vec![],
            },
            true,
            FakeUserMembershipPort {
                tier: Some("V5".to_string()),
                open_id: None,
            },
        );

        let error = use_case
            .execute(input())
            .await
            .expect_err("wechat binding required");

        assert!(matches!(
            downcast_error(&error),
            MatchIdUnlockError::WechatBindingRequired
        ));
        assert_eq!(
            order_repository
                .created_orders
                .lock()
                .expect("created orders")
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn execute_rejects_missing_match() {
        let (use_case, _, _) = use_case(
            FakeRepository {
                unlocked_matches: vec![],
            },
            false,
            FakeUserMembershipPort {
                tier: Some("V5".to_string()),
                open_id: Some("openid".to_string()),
            },
        );

        let error = use_case
            .execute(input())
            .await
            .expect_err("match should be missing");

        assert!(matches!(
            downcast_error(&error),
            MatchIdUnlockError::MatchNotFound
        ));
    }
}
