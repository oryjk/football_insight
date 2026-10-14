use crate::admin::{
    domain::admin_product::{
        AdminProduct, AdminProductError, AdminProductPage, AdminProductQuery, ValidatedAdminProduct,
    },
    ports::admin_product_repository::{AdminProductRepository, ProductAudit},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

const COLUMNS: &str = "id AS catalog_product_id, product_key, split_part(split_part(product_key, '@', 1), ':', 2) AS membership_tier, name, product_id, price_cents, membership_duration_months, enabled, remark, allowed_user_ids, created_at, updated_at";
const SCOPE: &str = "product_key ~ '^membership:V(3|[6-9])@[1-9][0-9]*$'";
#[derive(sqlx::FromRow)]
struct ProductRow {
    catalog_product_id: Uuid,
    product_key: String,
    membership_tier: String,
    name: String,
    product_id: String,
    price_cents: i32,
    membership_duration_months: i32,
    enabled: bool,
    remark: String,
    allowed_user_ids: Vec<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
impl From<ProductRow> for AdminProduct {
    fn from(v: ProductRow) -> Self {
        Self {
            catalog_product_id: v.catalog_product_id,
            product_key: v.product_key,
            membership_tier: v.membership_tier,
            name: v.name,
            product_id: v.product_id,
            price_cents: v.price_cents,
            membership_duration_months: v.membership_duration_months,
            enabled: v.enabled,
            remark: v.remark,
            allowed_user_ids: v.allowed_user_ids,
            created_at: v.created_at,
            updated_at: v.updated_at,
        }
    }
}
pub struct PostgresAdminProductRepository {
    pool: PgPool,
}
impl PostgresAdminProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl AdminProductRepository for PostgresAdminProductRepository {
    async fn members(
        &self,
        key: &str,
        page: u32,
        page_size: u32,
    ) -> Result<crate::admin::domain::admin_product_purchase::ProductMemberPage, AdminProductError>
    {
        use crate::admin::domain::admin_product_purchase::{ProductMember, ProductMemberPage};
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(map_error)?;
        let (id, tier) = sqlx::query_as::<_, (Uuid, String)>(&format!("SELECT id,split_part(split_part(product_key,'@',1),':',2) FROM f_i_virtual_payment_products WHERE product_key=$1 AND {SCOPE}"))
            .bind(key).fetch_optional(&mut *tx).await.map_err(map_error)?.ok_or(AdminProductError::NotFound)?;
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM f_i_users WHERE membership_tier=$1 AND status='active'",
        )
        .bind(&tier)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_error)?;
        let rows = sqlx::query_as::<_, (Uuid, String, Option<String>, String, Option<DateTime<Utc>>, bool)>("SELECT id,COALESCE(NULLIF(display_name,''),'未命名用户'),avatar_url,membership_origin,membership_expires_at,COALESCE(membership_expires_at<=NOW(),FALSE) FROM f_i_users WHERE membership_tier=$1 AND status='active' ORDER BY created_at DESC,id ASC LIMIT $2 OFFSET $3")
            .bind(&tier).bind(i64::from(page_size)).bind(i64::from(page-1)*i64::from(page_size)).fetch_all(&mut *tx).await.map_err(map_error)?;
        tx.commit().await.map_err(map_error)?;
        Ok(ProductMemberPage {
            catalog_product_id: id,
            membership_tier: tier,
            total,
            page,
            page_size,
            items: rows
                .into_iter()
                .map(
                    |(
                        user_id,
                        user_nickname,
                        user_avatar_url,
                        membership_origin,
                        expires_at,
                        expired,
                    )| ProductMember {
                        user_id,
                        user_nickname,
                        user_avatar_url,
                        membership_origin,
                        expires_at,
                        expired,
                    },
                )
                .collect(),
        })
    }
    async fn purchases(
        &self,
        key: &str,
        page: u32,
        page_size: u32,
    ) -> Result<crate::admin::domain::admin_product_purchase::ProductPurchasePage, AdminProductError>
    {
        use crate::admin::domain::admin_product_purchase::{
            ProductPurchasePage, ProductPurchaseSummary,
        };
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(map_error)?;
        let id = sqlx::query_scalar::<_, Uuid>(&format!(
            "SELECT id FROM f_i_virtual_payment_products WHERE product_key=$1 AND {SCOPE}"
        ))
        .bind(key)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_error)?
        .ok_or(AdminProductError::NotFound)?;
        let counts=sqlx::query_as::<_,(i64,i64,i64,i64)>("SELECT COUNT(DISTINCT user_id),COUNT(*),COUNT(*) FILTER(WHERE status='refunded'),COALESCE(SUM(amount_cents),0)::bigint FROM f_i_payment_orders WHERE catalog_product_id=$1 AND paid_at IS NOT NULL AND status IN ('paid','refunded')")
            .bind(id).fetch_one(&mut *tx).await.map_err(map_error)?;
        let rows=sqlx::query_as::<_,PurchaseRow>("SELECT p.order_no,p.user_id,COALESCE(NULLIF(u.display_name,''),'未命名用户') AS user_nickname,u.avatar_url AS user_avatar_url,p.paid_at,p.amount_cents,p.status,(v.order_no IS NOT NULL) AS has_virtual,v.grant_after FROM f_i_payment_orders p LEFT JOIN f_i_users u ON u.id=p.user_id LEFT JOIN f_i_virtual_payment_orders v ON v.order_no=p.order_no WHERE p.catalog_product_id=$1 AND p.paid_at IS NOT NULL AND p.status IN ('paid','refunded') ORDER BY p.paid_at DESC,p.order_no ASC LIMIT $2 OFFSET $3")
            .bind(id).bind(i64::from(page_size)).bind(i64::from(page-1)*i64::from(page_size)).fetch_all(&mut *tx).await.map_err(map_error)?;
        tx.commit().await.map_err(map_error)?;
        Ok(ProductPurchasePage {
            catalog_product_id: id,
            summary: ProductPurchaseSummary {
                purchased_user_count: counts.0,
                purchased_order_count: counts.1,
                refunded_order_count: counts.2,
                total_amount_cents: counts.3,
            },
            items: rows.into_iter().map(Into::into).collect(),
            total: counts.1,
            page,
            page_size,
        })
    }
    async fn list(&self, query: AdminProductQuery) -> Result<AdminProductPage, AdminProductError> {
        let search = query.query.as_ref().map(|v| {
            format!(
                "%{}%",
                v.replace('\\', "\\\\")
                    .replace('%', "\\%")
                    .replace('_', "\\_")
            )
        });
        let predicate = format!(
            "{SCOPE} AND ($1::text IS NULL OR product_key ILIKE $1 OR name ILIKE $1 OR product_id ILIKE $1 OR remark ILIKE $1) AND ($2::text IS NULL OR split_part(split_part(product_key, '@', 1), ':', 2) = $2) AND ($3::boolean IS NULL OR enabled = $3)"
        );
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(map_error)?;
        let total = sqlx::query_scalar::<_, i64>(&format!(
            "SELECT COUNT(*) FROM f_i_virtual_payment_products WHERE {predicate}"
        ))
        .bind(&search)
        .bind(&query.membership_tier)
        .bind(query.enabled)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_error)?;
        let rows=sqlx::query_as::<_,ProductRow>(&format!("SELECT {COLUMNS} FROM f_i_virtual_payment_products WHERE {predicate} ORDER BY updated_at DESC, product_key ASC LIMIT $4 OFFSET $5"))
            .bind(&search).bind(&query.membership_tier).bind(query.enabled).bind(i64::from(query.page_size)).bind(i64::from(query.page-1)*i64::from(query.page_size)).fetch_all(&mut *tx).await.map_err(map_error)?;
        tx.commit().await.map_err(map_error)?;
        Ok(AdminProductPage {
            items: rows.into_iter().map(Into::into).collect(),
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }
    async fn get(&self, key: &str) -> Result<AdminProduct, AdminProductError> {
        sqlx::query_as::<_, ProductRow>(&format!(
            "SELECT {COLUMNS} FROM f_i_virtual_payment_products WHERE product_key=$1 AND {SCOPE}"
        ))
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?
        .map(Into::into)
        .ok_or(AdminProductError::NotFound)
    }
    async fn create_with_audit(
        &self,
        input: ValidatedAdminProduct,
        audit: ProductAudit,
    ) -> Result<AdminProduct, AdminProductError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let row=sqlx::query_as::<_,ProductRow>(&format!("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents,enabled,remark,allowed_user_ids,membership_duration_months) VALUES($1,$2,$3,$4,$5,$6,$7,$8) RETURNING {COLUMNS}"))
            .bind(&input.product_key).bind(&input.name).bind(&input.product_id).bind(input.price_cents).bind(input.enabled).bind(&input.remark).bind(&input.allowed_user_ids).bind(input.membership_duration_months).fetch_one(&mut *tx).await.map_err(map_error)?;
        let product: AdminProduct = row.into();
        insert_audit(&mut tx, audit, &product.product_key, None, Some(&product)).await?;
        tx.commit().await.map_err(map_error)?;
        Ok(product)
    }
    async fn update_with_audit(
        &self,
        key: &str,
        input: ValidatedAdminProduct,
        audit: ProductAudit,
    ) -> Result<AdminProduct, AdminProductError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let before = lock_product(&mut tx, key).await?;
        let row=sqlx::query_as::<_,ProductRow>(&format!("UPDATE f_i_virtual_payment_products SET product_key=$2,name=$3,product_id=$4,price_cents=$5,enabled=$6,remark=$7,allowed_user_ids=$8,membership_duration_months=$9 WHERE product_key=$1 RETURNING {COLUMNS}"))
            .bind(key).bind(&input.product_key).bind(&input.name).bind(&input.product_id).bind(input.price_cents).bind(input.enabled).bind(&input.remark).bind(&input.allowed_user_ids).bind(input.membership_duration_months).fetch_one(&mut *tx).await.map_err(map_error)?;
        let product: AdminProduct = row.into();
        insert_audit(
            &mut tx,
            audit,
            &product.product_key,
            Some(&before),
            Some(&product),
        )
        .await?;
        tx.commit().await.map_err(map_error)?;
        Ok(product)
    }
    async fn delete_with_audit(
        &self,
        key: &str,
        audit: ProductAudit,
    ) -> Result<(), AdminProductError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let before = lock_product(&mut tx, key).await?;
        sqlx::query("DELETE FROM f_i_virtual_payment_products WHERE product_key=$1")
            .bind(key)
            .execute(&mut *tx)
            .await
            .map_err(map_error)?;
        insert_audit(&mut tx, audit, key, Some(&before), None).await?;
        tx.commit().await.map_err(map_error)?;
        Ok(())
    }
}
async fn lock_product(
    tx: &mut Transaction<'_, Postgres>,
    key: &str,
) -> Result<AdminProduct, AdminProductError> {
    sqlx::query_as::<_,ProductRow>(&format!("SELECT {COLUMNS} FROM f_i_virtual_payment_products WHERE product_key=$1 AND {SCOPE} FOR UPDATE"))
        .bind(key).fetch_optional(&mut **tx).await.map_err(map_error)?.map(Into::into).ok_or(AdminProductError::NotFound)
}
async fn insert_audit(
    tx: &mut Transaction<'_, Postgres>,
    audit: ProductAudit,
    key: &str,
    before: Option<&AdminProduct>,
    after: Option<&AdminProduct>,
) -> Result<(), AdminProductError> {
    sqlx::query("INSERT INTO f_i_admin_audit_logs(id,admin_user_id,action,target_type,target_id,before_json,after_json) VALUES($1,$2,$3,'product',$4,$5,$6)")
        .bind(Uuid::new_v4()).bind(audit.admin_id).bind(audit.action).bind(key).bind(before.map(snapshot)).bind(after.map(snapshot)).execute(&mut **tx).await.map_err(map_error)?;
    Ok(())
}
fn snapshot(v: &AdminProduct) -> serde_json::Value {
    serde_json::json!({"catalog_product_id":v.catalog_product_id,"product_key":v.product_key,"membership_tier":v.membership_tier,"name":v.name,"product_id":v.product_id,"price_cents":v.price_cents,"membership_duration_months":v.membership_duration_months,"enabled":v.enabled,"remark":v.remark,"allowed_user_ids":v.allowed_user_ids,"created_at":v.created_at,"updated_at":v.updated_at})
}
fn map_error(error: sqlx::Error) -> AdminProductError {
    if error
        .as_database_error()
        .is_some_and(|e| e.is_unique_violation())
    {
        return AdminProductError::Conflict;
    }
    tracing::error!(error=%error,"admin product persistence failed");
    AdminProductError::Internal
}

#[derive(sqlx::FromRow)]
struct PurchaseRow {
    order_no: String,
    user_id: Uuid,
    user_nickname: String,
    user_avatar_url: Option<String>,
    paid_at: DateTime<Utc>,
    amount_cents: i32,
    status: String,
    has_virtual: bool,
    grant_after: Option<serde_json::Value>,
}
impl From<PurchaseRow> for crate::admin::domain::admin_product_purchase::ProductPurchase {
    fn from(row: PurchaseRow) -> Self {
        let snapshot = row
            .grant_after
            .as_ref()
            .and_then(|value| value.get("expires_at"))
            .and_then(|value| match value {
                serde_json::Value::Null => Some(None),
                serde_json::Value::String(value) => DateTime::parse_from_rfc3339(value)
                    .ok()
                    .map(|value| Some(value.with_timezone(&Utc))),
                _ => None,
            });
        let (expires_at, expiry_source) =
            crate::admin::domain::admin_product_purchase::resolve_purchase_expiry(
                row.paid_at,
                row.has_virtual,
                snapshot,
            );
        Self {
            order_no: row.order_no,
            user_id: row.user_id,
            user_nickname: row.user_nickname,
            user_avatar_url: row.user_avatar_url,
            paid_at: row.paid_at,
            expires_at,
            expiry_source,
            amount_cents: row.amount_cents,
            status: row.status,
        }
    }
}
