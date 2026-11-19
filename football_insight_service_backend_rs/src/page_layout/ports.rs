use async_trait::async_trait;
use uuid::Uuid;

use super::domain::{
    PageLayoutError, ParsedLayoutImage, StoredPageLayout, ValidatedPageLayoutWrite,
};

#[async_trait]
pub trait PageLayoutRepository: Send + Sync {
    async fn list_all(&self) -> Result<Vec<StoredPageLayout>, PageLayoutError>;
    async fn get_by_page_key(
        &self,
        page_key: &str,
    ) -> Result<Option<StoredPageLayout>, PageLayoutError>;
    async fn upsert(
        &self,
        write: ValidatedPageLayoutWrite,
        admin_id: Uuid,
    ) -> Result<StoredPageLayout, PageLayoutError>;
    async fn reset_to_default(
        &self,
        page_key: &str,
        admin_id: Uuid,
    ) -> Result<(), PageLayoutError>;
}

#[async_trait]
pub trait LayoutImageStoragePort: Send + Sync {
    async fn upload_image(&self, image: ParsedLayoutImage) -> Result<String, PageLayoutError>;
}
