use async_trait::async_trait;
use uuid::Uuid;

use crate::admin::domain::admin_brand::{AdminBrandError, BrandSubtitle};

#[async_trait]
pub trait AdminBrandRepository: Send + Sync {
    async fn get_subtitle(&self) -> Result<BrandSubtitle, AdminBrandError>;
    /// 写入并记审计；空值删除配置行，恢复客户端默认文案。
    async fn set_subtitle(
        &self,
        subtitle: BrandSubtitle,
        admin_id: Uuid,
    ) -> Result<BrandSubtitle, AdminBrandError>;
}
