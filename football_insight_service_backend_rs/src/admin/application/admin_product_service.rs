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
    pub fn new(repository: Arc<dyn AdminProductRepository>) -> Self {
        Self { repository }
    }
    pub async fn list(
        &self,
        query: AdminProductQuery,
    ) -> Result<AdminProductPage, AdminProductError> {
        self.repository.list(query.validate()?).await
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
