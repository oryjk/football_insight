use super::{
    domain::{ServiceCode, StudentConfig, StudentError, StudentOrderPage},
    ports::StudentRepository,
};
use crate::{
    auth::ports::user_membership_port::UserMembershipPort,
    payment::{
        domain::order::{NewPaymentOrder, WxPayParams},
        ports::{order_repository::OrderRepository, wechat_pay_port::WechatPayPort},
    },
};
use std::sync::Arc;
use uuid::Uuid;

pub struct StudentService {
    repository: Arc<dyn StudentRepository>,
    orders: Arc<dyn OrderRepository>,
    users: Arc<dyn UserMembershipPort>,
    pay: Arc<dyn WechatPayPort>,
}
pub struct StudentConfigService {
    repository: Arc<dyn StudentRepository>,
}
impl StudentConfigService {
    pub fn new(repository: Arc<dyn StudentRepository>) -> Self {
        Self { repository }
    }
    pub async fn config(&self) -> Result<StudentConfig, StudentError> {
        self.repository.config().await
    }
    pub async fn save(
        &self,
        config: StudentConfig,
        admin: Uuid,
    ) -> Result<StudentConfig, StudentError> {
        self.repository.save_config(config.validate()?, admin).await
    }
}
impl StudentService {
    pub fn new(
        repository: Arc<dyn StudentRepository>,
        orders: Arc<dyn OrderRepository>,
        users: Arc<dyn UserMembershipPort>,
        pay: Arc<dyn WechatPayPort>,
    ) -> Self {
        Self {
            repository,
            orders,
            users,
            pay,
        }
    }
    pub async fn config(&self) -> Result<StudentConfig, StudentError> {
        self.repository.config().await
    }
    pub async fn orders(
        &self,
        user: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<StudentOrderPage, StudentError> {
        if !(1..=1_000_000).contains(&page) || !(1..=100).contains(&page_size) {
            return Err(StudentError::InvalidInput("分页参数无效".into()));
        }
        self.repository.orders(user, page, page_size).await
    }
    pub async fn create_order(
        &self,
        user: Uuid,
        code: ServiceCode,
    ) -> Result<StudentCheckout, StudentError> {
        let config = self.repository.config().await?.validate()?;
        let item = config
            .services
            .iter()
            .find(|item| item.code == code && item.enabled)
            .ok_or(StudentError::Unavailable)?;
        let open_id = self
            .users
            .get_user_open_id(user)
            .await
            .map_err(|e| internal("student user identity", e))?
            .ok_or(StudentError::WechatRequired)?;
        let order_no = format!("S{}", Uuid::new_v4().simple());
        // Provider permits at most 32 ASCII characters.
        let order_no = order_no[..32].to_owned();
        self.orders
            .create_order(NewPaymentOrder {
                order_no: order_no.clone(),
                user_id: user,
                amount_cents: code.price(),
                product_type: code.product_type(),
            })
            .await
            .map_err(|e| internal("student create order", e))?;
        let params_result = self
            .pay
            .unified_order(&order_no, &item.title, code.price(), &open_id)
            .await;
        let params = match params_result {
            Ok(params) if params.virtual_payment.is_some() => params,
            result => {
                if let Err(error) = result {
                    tracing::warn!(%error, "student payment parameters failed");
                }
                // A recorded virtual order may already be reconcilable; never cancel it here.
                self.repository.fail_unissued_order(user, &order_no).await?;
                return Err(StudentError::PaymentFailed);
            }
        };
        Ok(StudentCheckout { order_no, params })
    }
}
pub struct StudentCheckout {
    pub order_no: String,
    pub params: WxPayParams,
}
fn internal(context: &str, error: anyhow::Error) -> StudentError {
    tracing::warn!(%context, %error, "student operation failed");
    StudentError::Internal
}
