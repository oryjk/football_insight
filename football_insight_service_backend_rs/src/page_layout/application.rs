use std::collections::HashMap;
use std::sync::Arc;

use uuid::Uuid;

use super::{
    domain::{
        PAGE_LAYOUT_CATALOG, PageLayoutError, PublicPageLayout, ResolvedPageLayout,
        SavePageLayoutInput, find_page_definition, parse_layout_image_data_url,
        resolve_page_layout, validate_save_input,
    },
    ports::{LayoutImageStoragePort, PageLayoutRepository},
};

pub struct PageLayoutService {
    repository: Arc<dyn PageLayoutRepository>,
    image_storage: Arc<dyn LayoutImageStoragePort>,
}

impl PageLayoutService {
    pub fn new(
        repository: Arc<dyn PageLayoutRepository>,
        image_storage: Arc<dyn LayoutImageStoragePort>,
    ) -> Self {
        Self {
            repository,
            image_storage,
        }
    }

    pub async fn list_admin_layouts(&self) -> Result<Vec<ResolvedPageLayout>, PageLayoutError> {
        let stored_items = self.repository.list_all().await?;
        let by_key: HashMap<&str, _> = stored_items
            .iter()
            .map(|item| (item.page_key.as_str(), item))
            .collect();
        Ok(PAGE_LAYOUT_CATALOG
            .iter()
            .map(|def| resolve_page_layout(def, by_key.get(def.page_key).copied()))
            .collect())
    }

    pub async fn list_public_layouts(&self) -> Result<Vec<PublicPageLayout>, PageLayoutError> {
        let resolved = self.list_admin_layouts().await?;
        Ok(resolved.into_iter().map(PublicPageLayout::from).collect())
    }

    pub async fn get_public_layout(
        &self,
        page_key: &str,
    ) -> Result<PublicPageLayout, PageLayoutError> {
        let def = find_page_definition(page_key).ok_or(PageLayoutError::NotFound)?;
        let stored = self.repository.get_by_page_key(def.page_key).await?;
        Ok(PublicPageLayout::from(resolve_page_layout(
            def,
            stored.as_ref(),
        )))
    }

    pub async fn save_layout(
        &self,
        page_key: &str,
        input: SavePageLayoutInput,
        admin_id: Uuid,
    ) -> Result<ResolvedPageLayout, PageLayoutError> {
        let def = find_page_definition(page_key).ok_or(PageLayoutError::NotFound)?;
        let validated = validate_save_input(def.page_key, input)?;
        let saved = self.repository.upsert(validated, admin_id).await?;
        Ok(resolve_page_layout(def, Some(&saved)))
    }

    pub async fn reset_layout(
        &self,
        page_key: &str,
        admin_id: Uuid,
    ) -> Result<ResolvedPageLayout, PageLayoutError> {
        let def = find_page_definition(page_key).ok_or(PageLayoutError::NotFound)?;
        self.repository
            .reset_to_default(def.page_key, admin_id)
            .await?;
        Ok(resolve_page_layout(def, None))
    }

    pub async fn upload_image(&self, data_url: &str) -> Result<String, PageLayoutError> {
        let parsed = parse_layout_image_data_url(data_url)?;
        self.image_storage.upload_image(parsed).await
    }
}
