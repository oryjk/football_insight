use crate::payment::{
    domain::virtual_payment::{VirtualPaymentError, VirtualProduct},
    ports::virtual_payment_port::VirtualProductRepository,
};
use sqlx::PgPool;

pub struct PostgresVirtualProductRepository {
    pool: PgPool,
}

impl PostgresVirtualProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn catalog_error(error: sqlx::Error) -> VirtualPaymentError {
    tracing::error!(%error, "failed to read virtual payment product catalog");
    VirtualPaymentError::CatalogUnavailable
}

#[async_trait::async_trait]
impl VirtualProductRepository for PostgresVirtualProductRepository {
    async fn find_enabled(&self, key: &str) -> Result<Option<VirtualProduct>, VirtualPaymentError> {
        // Do not cache: edits/disablement must affect the next purchase without a restart.
        let row = sqlx::query_as::<_, (String, i32)>(
            "SELECT product_id, price_cents FROM f_i_virtual_payment_products WHERE product_key=$1 AND enabled",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(catalog_error)?;
        Ok(row.map(|(product_id, price_cents)| VirtualProduct {
            product_id,
            price_cents,
        }))
    }

    async fn has_enabled(&self) -> Result<bool, VirtualPaymentError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM f_i_virtual_payment_products WHERE enabled)",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(catalog_error)
    }
}
