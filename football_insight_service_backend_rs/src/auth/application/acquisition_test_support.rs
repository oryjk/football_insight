use crate::auth::{
    domain::acquisition::AcquisitionSource, ports::acquisition_repository::AcquisitionRepository,
};
use async_trait::async_trait;
use std::sync::Mutex;
use uuid::Uuid;
#[derive(Default)]
pub(super) struct Sources(pub Mutex<Vec<(Uuid, AcquisitionSource)>>);
#[async_trait]
impl AcquisitionRepository for Sources {
    async fn insert_first(&self, user_id: Uuid, source: AcquisitionSource) -> anyhow::Result<()> {
        self.0.lock().unwrap().push((user_id, source));
        Ok(())
    }
}
