use crate::payment::{
    domain::virtual_payment::{PrivateMembershipProduct, VirtualPaymentError, VirtualProduct},
    ports::virtual_payment_port::VirtualProductRepository,
};
use sqlx::{PgPool, Row};

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
    async fn membership_catalog_products(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<crate::payment::domain::product_kind::ProductSpec>, VirtualPaymentError> {
        let rows=sqlx::query("SELECT id,product_key,kind,price_cents,membership_duration_months,grant_spec,pricing_rule,eligibility,requires_capability,title,subtitle,description,sort_order,group_key FROM f_i_virtual_payment_products WHERE enabled AND product_key LIKE 'membership:%' AND (cardinality(allowed_user_ids)=0 OR $1=ANY(allowed_user_ids)) ORDER BY sort_order,product_key")
            .bind(user_id).fetch_all(&self.pool).await.map_err(catalog_error)?;
        rows.into_iter().map(|row| {
            let mut spec=crate::payment::domain::product_kind::ProductSpec {
                catalog_product_id:row.try_get("id")?,product_key:row.try_get("product_key")?,kind:row.try_get("kind")?,price_cents:row.try_get("price_cents")?,membership_duration_months:row.try_get("membership_duration_months")?,grant_spec:row.try_get("grant_spec")?,pricing_rule:row.try_get("pricing_rule")?,eligibility:row.try_get("eligibility")?,requires_capability:row.try_get("requires_capability")?,title:row.try_get("title")?,subtitle:row.try_get("subtitle")?,description:row.try_get("description")?,sort_order:row.try_get("sort_order")?,group_key:row.try_get("group_key")?,
            };
            // Legacy admin edits still maintain the legacy duration/key columns during P1.
            if !spec.is_table_offer() && spec.kind=="membership" {
                let tier=spec.product_key.split('@').next().unwrap_or("").strip_prefix("membership:").unwrap_or("");
                spec.grant_spec=serde_json::json!({"tier":tier,"expiry":{"rule":"months","months":spec.membership_duration_months}});
            }
            Ok(spec)
        }).collect::<Result<Vec<_>,sqlx::Error>>().map_err(catalog_error)
    }

    async fn find_enabled(
        &self,
        key: &str,
        user_id: uuid::Uuid,
    ) -> Result<Option<VirtualProduct>, VirtualPaymentError> {
        // Do not cache: edits/disablement must affect the next purchase without a restart.
        let row = sqlx::query_as::<_, (uuid::Uuid,String, i32, i32)>(
            "SELECT id, product_id, price_cents, membership_duration_months FROM f_i_virtual_payment_products WHERE product_key=$1 AND enabled AND (cardinality(allowed_user_ids)=0 OR $2=ANY(allowed_user_ids))",
        )
        .bind(key)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(catalog_error)?;
        Ok(row.map(
            |(catalog_id, product_id, price_cents, membership_duration_months)| VirtualProduct {
                catalog_product_id: Some(catalog_id),
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
            "SELECT split_part(product_key,'@',1),name,price_cents,membership_duration_months FROM f_i_virtual_payment_products WHERE product_key ~ '^membership:V[3-9]@[1-9][0-9]*$' AND requires_capability IS NULL AND enabled AND cardinality(allowed_user_ids)>0 AND $1=ANY(allowed_user_ids) ORDER BY price_cents,product_key",
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
