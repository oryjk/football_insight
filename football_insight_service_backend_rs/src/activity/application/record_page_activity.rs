use std::sync::Arc;

use uuid::Uuid;

use crate::activity::{
    domain::page_key::validate_activity_page_key,
    ports::user_activity_repository::UserActivityRepository,
};

#[derive(Debug, Clone)]
pub struct RecordPageActivityInput {
    pub user_id: Uuid,
    pub page_key: String,
}

pub struct RecordPageActivityUseCase {
    repository: Arc<dyn UserActivityRepository>,
}

impl RecordPageActivityUseCase {
    pub fn new(repository: Arc<dyn UserActivityRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: RecordPageActivityInput) -> anyhow::Result<()> {
        let page_key = validate_activity_page_key(&input.page_key)?;
        self.repository
            .record_page_activity(input.user_id, &page_key)
            .await
    }
}
