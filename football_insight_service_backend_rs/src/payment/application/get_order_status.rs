use std::sync::Arc;

use crate::payment::{domain::order::PaymentOrder, ports::order_repository::OrderRepository};

pub struct GetOrderStatusUseCase {
    order_repository: Arc<dyn OrderRepository>,
}

impl GetOrderStatusUseCase {
    pub fn new(order_repository: Arc<dyn OrderRepository>) -> Self {
        Self { order_repository }
    }

    pub async fn execute(
        &self,
        input: GetOrderStatusInput,
    ) -> anyhow::Result<Option<PaymentOrder>> {
        let order = self
            .order_repository
            .find_order_by_no(&input.order_no)
            .await?;

        if let Some(ref order) = order {
            if order.user_id != input.user_id {
                anyhow::bail!("无权查看此订单");
            }
        }

        Ok(order)
    }
}

pub struct GetOrderStatusInput {
    pub order_no: String,
    pub user_id: uuid::Uuid,
}
