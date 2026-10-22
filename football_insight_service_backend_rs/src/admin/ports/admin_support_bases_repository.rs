use async_trait::async_trait;

use crate::admin::domain::admin_support_bases::{
    AdminSupportBasesError, SupportBaseAudit, SupportBaseInput, SupportBaseMatch,
};

#[async_trait]
pub trait AdminSupportBasesRepository: Send + Sync {
    /// Recent and upcoming matches with their real counts and current bases.
    async fn list_matches(
        &self,
        limit: i64,
    ) -> Result<Vec<SupportBaseMatch>, AdminSupportBasesError>;
    async fn set_bases(
        &self,
        match_id: i64,
        input: SupportBaseInput,
        audit: SupportBaseAudit,
    ) -> Result<SupportBaseMatch, AdminSupportBasesError>;
}
