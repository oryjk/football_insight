use crate::admin::domain::admin_product::{
    AdminProduct, AdminProductError, AdminProductPage, AdminProductQuery, ValidatedAdminProduct,
};
use async_trait::async_trait;
use uuid::Uuid;
#[derive(Debug, Clone, Copy)]
pub struct ProductAudit {
    pub admin_id: Uuid,
    pub action: &'static str,
}
/// Every mutation and its before/after audit must commit or roll back together.
#[async_trait]
pub trait AdminProductRepository: Send + Sync {
    async fn members(
        &self,
        key: &str,
        page: u32,
        page_size: u32,
        origins: Vec<String>,
    ) -> Result<crate::admin::domain::admin_product_purchase::ProductMemberPage, AdminProductError>;
    async fn purchases(
        &self,
        key: &str,
        page: u32,
        page_size: u32,
    ) -> Result<crate::admin::domain::admin_product_purchase::ProductPurchasePage, AdminProductError>;
    async fn list(&self, query: AdminProductQuery) -> Result<AdminProductPage, AdminProductError>;
    async fn get(&self, key: &str) -> Result<AdminProduct, AdminProductError>;
    async fn create_with_audit(
        &self,
        input: ValidatedAdminProduct,
        audit: ProductAudit,
    ) -> Result<AdminProduct, AdminProductError>;
    async fn update_with_audit(
        &self,
        key: &str,
        input: ValidatedAdminProduct,
        audit: ProductAudit,
    ) -> Result<AdminProduct, AdminProductError>;
    async fn delete_with_audit(
        &self,
        key: &str,
        audit: ProductAudit,
    ) -> Result<(), AdminProductError>;
}
