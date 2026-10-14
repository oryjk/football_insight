use crate::admin::{
    domain::admin_product::{
        AdminProduct, AdminProductError, AdminProductInput, AdminProductPage, AdminProductQuery,
        validate_product_key,
    },
    ports::admin_product_repository::{AdminProductRepository, ProductAudit},
};
use std::sync::Arc;
use uuid::Uuid;
pub struct AdminProductService {
    repository: Arc<dyn AdminProductRepository>,
}
impl AdminProductService {
    pub async fn members(
        &self,
        key: &str,
        page: u32,
        page_size: u32,
        origins: Option<Vec<String>>,
    ) -> Result<crate::admin::domain::admin_product_purchase::ProductMemberPage, AdminProductError>
    {
        validate_product_key(key)?;
        let query = AdminProductQuery {
            page,
            page_size,
            ..Default::default()
        }
        .validate()?;
        let mut origins =
            origins.unwrap_or_else(|| vec!["purchase".into(), "referral".into(), "unknown".into()]);
        if origins
            .iter()
            .any(|origin| !matches!(origin.as_str(), "purchase" | "referral" | "unknown"))
        {
            return Err(AdminProductError::InvalidInput(
                "invalid membership origins".into(),
            ));
        }
        origins.sort();
        origins.dedup();
        self.repository
            .members(key, query.page, query.page_size, origins)
            .await
    }
    pub fn new(repository: Arc<dyn AdminProductRepository>) -> Self {
        Self { repository }
    }
    pub async fn list(
        &self,
        query: AdminProductQuery,
    ) -> Result<AdminProductPage, AdminProductError> {
        self.repository.list(query.validate()?).await
    }
    pub async fn purchases(
        &self,
        key: &str,
        page: u32,
        page_size: u32,
    ) -> Result<crate::admin::domain::admin_product_purchase::ProductPurchasePage, AdminProductError>
    {
        validate_product_key(key)?;
        let query = AdminProductQuery {
            page,
            page_size,
            ..Default::default()
        }
        .validate()?;
        self.repository
            .purchases(key, query.page, query.page_size)
            .await
    }
    pub async fn get(&self, key: &str) -> Result<AdminProduct, AdminProductError> {
        validate_product_key(key)?;
        self.repository.get(key).await
    }
    pub async fn create(
        &self,
        input: AdminProductInput,
        actor: Uuid,
    ) -> Result<AdminProduct, AdminProductError> {
        self.repository
            .create_with_audit(
                input.validate()?,
                ProductAudit {
                    admin_id: actor,
                    action: "product.create",
                },
            )
            .await
    }
    pub async fn update(
        &self,
        key: &str,
        input: AdminProductInput,
        actor: Uuid,
    ) -> Result<AdminProduct, AdminProductError> {
        validate_product_key(key)?;
        self.repository
            .update_with_audit(
                key,
                input.validate()?,
                ProductAudit {
                    admin_id: actor,
                    action: "product.update",
                },
            )
            .await
    }
    pub async fn delete(&self, key: &str, actor: Uuid) -> Result<(), AdminProductError> {
        validate_product_key(key)?;
        self.repository
            .delete_with_audit(
                key,
                ProductAudit {
                    admin_id: actor,
                    action: "product.delete",
                },
            )
            .await
    }
}
