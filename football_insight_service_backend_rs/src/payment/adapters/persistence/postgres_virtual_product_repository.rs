use crate::payment::{
    domain::virtual_payment::{PrivateMembershipProduct, VirtualPaymentError, VirtualProduct},
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
    async fn find_enabled(
        &self,
        key: &str,
        user_id: uuid::Uuid,
    ) -> Result<Option<VirtualProduct>, VirtualPaymentError> {
        // Do not cache: edits/disablement must affect the next purchase without a restart.
        let row = sqlx::query_as::<_, (String, i32, i32)>(
            "SELECT product_id, price_cents, membership_duration_months FROM f_i_virtual_payment_products WHERE product_key=$1 AND enabled AND (cardinality(allowed_user_ids)=0 OR $2=ANY(allowed_user_ids))",
        )
        .bind(key)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(catalog_error)?;
        Ok(row.map(
            |(product_id, price_cents, membership_duration_months)| VirtualProduct {
                product_id,
                price_cents,
                membership_duration_months,
            },
        ))
    }

    async fn has_enabled(&self) -> Result<bool, VirtualPaymentError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM f_i_virtual_payment_products WHERE enabled)",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(catalog_error)
    }

    async fn private_membership_products(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<PrivateMembershipProduct>, VirtualPaymentError> {
        let rows = sqlx::query_as::<_, (String, String, i32, i32)>(
            "SELECT split_part(product_key,'@',1),name,price_cents,membership_duration_months FROM f_i_virtual_payment_products WHERE product_key LIKE 'membership:V3@%' AND enabled AND $1=ANY(allowed_user_ids) ORDER BY price_cents,product_key",
        ).bind(user_id).fetch_all(&self.pool).await.map_err(catalog_error)?;
        Ok(rows
            .into_iter()
            .map(
                |(product_type, name, price_cents, membership_duration_months)| {
                    PrivateMembershipProduct {
                        product_type,
                        name,
                        price_cents,
                        membership_duration_months,
                    }
                },
            )
            .collect())
    }
}
