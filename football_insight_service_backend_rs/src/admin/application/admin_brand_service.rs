use std::sync::Arc;

use uuid::Uuid;

use crate::admin::{
    domain::admin_brand::{AdminBrandError, BrandSubtitle},
    ports::admin_brand_repository::AdminBrandRepository,
};

pub struct AdminBrandService {
    repository: Arc<dyn AdminBrandRepository>,
}

impl AdminBrandService {
    pub fn new(repository: Arc<dyn AdminBrandRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_subtitle(&self) -> Result<BrandSubtitle, AdminBrandError> {
        self.repository.get_subtitle().await
    }

    pub async fn set_subtitle(
        &self,
        raw: &str,
        admin_id: Uuid,
    ) -> Result<BrandSubtitle, AdminBrandError> {
        let subtitle = BrandSubtitle::new(raw)?;
        self.repository.set_subtitle(subtitle, admin_id).await
    }
}
