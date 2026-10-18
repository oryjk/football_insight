use crate::auth::domain::acquisition::AcquisitionSource;
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait AcquisitionRepository: Send + Sync {
    async fn insert_first(&self, user_id: Uuid, source: AcquisitionSource) -> anyhow::Result<()>;
}
