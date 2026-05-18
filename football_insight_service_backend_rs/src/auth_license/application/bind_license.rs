use std::sync::Arc;

use crate::auth_license::ports::license_repository::LicenseRepository;

pub struct BindLicenseResult {
    pub user_id: uuid::Uuid,
    pub license_key: String,
}

pub struct BindLicenseUseCase {
    repository: Arc<dyn LicenseRepository>,
}

impl BindLicenseUseCase {
    pub fn new(repository: Arc<dyn LicenseRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, license_key: &str) -> anyhow::Result<BindLicenseResult> {
        let license = self
            .repository
            .find_by_key(license_key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("绑定码不存在"))?;
        if !license.is_valid() {
            return Err(anyhow::anyhow!("绑定码已失效或已使用"));
        }
        self.repository.mark_used(license.id).await?;
        Ok(BindLicenseResult {
            user_id: license.user_id,
            license_key: license.license_key,
        })
    }
}
