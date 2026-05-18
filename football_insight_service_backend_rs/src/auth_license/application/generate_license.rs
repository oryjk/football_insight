use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::auth_license::domain::license::{generate_license_key, UserLicense};
use crate::auth_license::ports::license_repository::LicenseRepository;

pub struct GenerateLicenseUseCase {
    repository: Arc<dyn LicenseRepository>,
}

impl GenerateLicenseUseCase {
    pub fn new(repository: Arc<dyn LicenseRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid) -> anyhow::Result<UserLicense> {
        let key = generate_license_key();
        let expires_at = Utc::now() + Duration::minutes(30);
        self.repository.create_license(user_id, &key, expires_at).await
    }
}
