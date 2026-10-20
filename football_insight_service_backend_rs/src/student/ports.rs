use super::domain::{StudentConfig, StudentError, StudentOrderPage};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait StudentRepository: Send + Sync {
    /// Failed checkout preparation is terminal only while no virtual payment parameters were issued.
    async fn fail_unissued_order(&self, user_id: Uuid, order_no: &str) -> Result<(), StudentError>;
    async fn config(&self) -> Result<StudentConfig, StudentError>;
    /// Config, live payment mappings and administrator audit are written atomically.
    async fn save_config(
        &self,
        config: StudentConfig,
        admin_id: Uuid,
    ) -> Result<StudentConfig, StudentError>;
    async fn orders(
        &self,
        user_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<StudentOrderPage, StudentError>;
}
