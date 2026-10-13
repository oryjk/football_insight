use crate::{
    auth::ports::user_membership_port::UserMembershipPort,
    payment::{
        application::handle_wechat_notify::HandleWechatNotifyUseCase,
        domain::{
            order::{OrderStatus, WxPayParams},
            virtual_payment::{
                Delivery, Refund, VirtualPaymentError, VirtualProduct, product_key,
                validate_delivery,
            },
        },
        ports::{
            order_repository::OrderRepository,
            virtual_payment_port::{VirtualOrderRepository, VirtualPaymentGateway},
            wechat_pay_port::WechatPayPort,
        },
    },
};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

pub struct VirtualPaymentService {
    gateway: Arc<dyn VirtualPaymentGateway>,
    orders: Arc<dyn OrderRepository>,
    virtual_orders: Arc<dyn VirtualOrderRepository>,
    users: Arc<dyn UserMembershipPort>,
    settlement: Arc<HandleWechatNotifyUseCase>,
    products: HashMap<String, VirtualProduct>,
}

impl VirtualPaymentService {
    pub fn new(
        gateway: Arc<dyn VirtualPaymentGateway>,
        orders: Arc<dyn OrderRepository>,
        virtual_orders: Arc<dyn VirtualOrderRepository>,
        users: Arc<dyn UserMembershipPort>,
        settlement: Arc<HandleWechatNotifyUseCase>,
        products: HashMap<String, VirtualProduct>,
    ) -> Self {
        Self {
            gateway,
            orders,
            virtual_orders,
            users,
            settlement,
            products,
        }
    }
    pub fn available(&self) -> bool {
        self.gateway.available() && !self.products.is_empty()
    }
    pub async fn prepare_session(&self, user_id: Uuid, code: &str) -> anyhow::Result<()> {
        if !self.available() {
            return Err(VirtualPaymentError::Unavailable.into());
        }
        let open_id = self
            .users
            .get_user_open_id(user_id)
            .await?
            .ok_or(VirtualPaymentError::IdentityMismatch)?;
        self.gateway.prepare_session(code, &open_id).await?;
        Ok(())
    }
    pub async fn deliver(&self, delivery: &Delivery) -> anyhow::Result<()> {
        let virtual_order = self
            .virtual_orders
            .find(&delivery.order_no)
            .await?
            .ok_or(VirtualPaymentError::InvalidOrder)?;
        validate_delivery(&virtual_order, delivery)?;
        // Persist the provider ID before settlement; uniqueness prevents it being reused on another order.
        self.virtual_orders.record_delivery(delivery).await?;
        let order = self
            .orders
            .find_order_by_no(&delivery.order_no)
            .await?
            .ok_or(VirtualPaymentError::InvalidOrder)?;
        if order.amount_cents != delivery.amount_cents {
            return Err(VirtualPaymentError::InvalidOrder.into());
        }
        if order.status == OrderStatus::Refunded {
            return Ok(());
        }
        if !matches!(order.status, OrderStatus::Pending | OrderStatus::Paid) {
            return Err(VirtualPaymentError::InvalidOrder.into());
        }
        self.settlement
            .settle_verified_order(&order, &delivery.wx_order_id)
            .await?;
        Ok(())
    }
    pub async fn deliver_callback(&self, mut delivery: Delivery) -> anyhow::Result<()> {
        let order = self
            .virtual_orders
            .find(&delivery.order_no)
            .await?
            .ok_or(VirtualPaymentError::InvalidOrder)?;
        // Query supplies the canonical provider ID for both WeChat Pay and Apple IAP.
        let proof = self
            .gateway
            .query_order(&order)
            .await?
            .ok_or(VirtualPaymentError::InvalidOrder)?;
        delivery.wx_order_id = proof.wx_order_id;
        self.deliver(&delivery).await?;
        self.acknowledge_delivery(&delivery.order_no).await
    }
    pub async fn refund(&self, refund: &Refund) -> anyhow::Result<()> {
        let order = self
            .virtual_orders
            .find(&refund.order_no)
            .await?
            .ok_or(VirtualPaymentError::InvalidOrder)?;
        if order.open_id != refund.open_id
            || order
                .wx_order_id
                .as_ref()
                .is_some_and(|id| id != &refund.wx_order_id)
            || refund.wx_order_id.is_empty()
            || refund.amount_cents <= 0
            || refund.amount_cents > order.amount_cents
            || refund.refund_id.is_empty()
        {
            return Err(VirtualPaymentError::InvalidOrder.into());
        }
        self.virtual_orders.refund(refund).await
    }
    pub async fn acknowledge_delivery(&self, order_no: &str) -> anyhow::Result<()> {
        self.virtual_orders.acknowledge_delivery(order_no).await
    }
    pub async fn reconcile(&self, order_no: &str, user_id: Option<Uuid>) -> anyhow::Result<()> {
        let base = self
            .orders
            .find_order_by_no(order_no)
            .await?
            .ok_or(VirtualPaymentError::InvalidOrder)?;
        if user_id.is_some_and(|id| id != base.user_id) {
            return Err(VirtualPaymentError::IdentityMismatch.into());
        }
        let Some(order) = self.virtual_orders.find(order_no).await? else {
            return Ok(());
        };
        if base.status == OrderStatus::Paid {
            self.gateway.confirm_delivery(&order).await?;
            self.virtual_orders.acknowledge_delivery(order_no).await?;
            return Ok(());
        }
        if base.status != OrderStatus::Pending {
            return Ok(());
        }
        if let Some(delivery) = self.gateway.query_order(&order).await? {
            self.deliver(&delivery).await?;
            self.gateway.confirm_delivery(&order).await?;
            self.virtual_orders.acknowledge_delivery(order_no).await?;
        }
        Ok(())
    }
    pub async fn reconcile_pending(&self) -> anyhow::Result<()> {
        if !self.available() {
            return Ok(());
        }
        for order in self.virtual_orders.pending_orders().await? {
            if let Err(error) = self.reconcile(&order.order_no, None).await {
                tracing::warn!(order_no = %order.order_no, error = %error, "virtual payment reconciliation failed");
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl WechatPayPort for VirtualPaymentService {
    async fn unified_order(
        &self,
        order_no: &str,
        _description: &str,
        amount_cents: i32,
        openid: &str,
    ) -> anyhow::Result<WxPayParams> {
        if !self.available() {
            return Err(VirtualPaymentError::Unavailable.into());
        }
        let order = self
            .orders
            .find_order_by_no(order_no)
            .await?
            .ok_or(VirtualPaymentError::InvalidOrder)?;
        if order.amount_cents != amount_cents || order.status != OrderStatus::Pending {
            return Err(VirtualPaymentError::InvalidOrder.into());
        }
        let key = product_key(&order.product_type, amount_cents)?;
        let product = self
            .products
            .get(&key)
            .ok_or(VirtualPaymentError::ProductUnavailable)?;
        if product.price_cents != amount_cents {
            return Err(VirtualPaymentError::ProductUnavailable.into());
        }
        let params = self
            .gateway
            .payment_params(order_no, openid, product)
            .await?;
        self.virtual_orders
            .record(crate::payment::domain::virtual_payment::VirtualOrder {
                order_no: order_no.into(),
                open_id: openid.into(),
                product_id: product.product_id.clone(),
                amount_cents,
                wx_order_id: None,
            })
            .await?;
        Ok(WxPayParams {
            virtual_payment: Some(params),
            time_stamp: String::new(),
            nonce_str: String::new(),
            package: String::new(),
            sign_type: String::new(),
            pay_sign: String::new(),
        })
    }
}
