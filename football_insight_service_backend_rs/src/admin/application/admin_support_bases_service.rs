use std::sync::Arc;

use uuid::Uuid;

use crate::admin::{
    domain::admin_support_bases::{
        AdminSupportBasesError, SupportBaseAudit, SupportBaseInput, SupportBaseMatch,
    },
    ports::admin_support_bases_repository::AdminSupportBasesRepository,
};

const MATCH_LIMIT: i64 = 40;

pub struct AdminSupportBasesService {
    repository: Arc<dyn AdminSupportBasesRepository>,
}

impl AdminSupportBasesService {
    pub fn new(repository: Arc<dyn AdminSupportBasesRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_matches(&self) -> Result<Vec<SupportBaseMatch>, AdminSupportBasesError> {
        self.repository.list_matches(MATCH_LIMIT).await
    }

    pub async fn set_bases(
        &self,
        match_id: i64,
        home_base: i64,
        away_base: i64,
        actor: Uuid,
    ) -> Result<SupportBaseMatch, AdminSupportBasesError> {
        if match_id <= 0 {
            return Err(AdminSupportBasesError::InvalidInput("invalid match_id".into()));
        }
        let input = SupportBaseInput::new(home_base, away_base)?;
        self.repository
            .set_bases(match_id, input, SupportBaseAudit { admin_id: actor })
            .await
    }
}
