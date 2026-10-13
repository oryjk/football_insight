use crate::admin::domain::admin_product::{
    AdminProduct, AdminProductError, AdminProductInput, AdminProductPage, AdminProductQuery,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminProductRequestDto {
    pub membership_tier: String,
    pub name: String,
    pub product_id: String,
    pub price_cents: i32,
    #[serde(default = "default_membership_duration_months")]
    pub membership_duration_months: i32,
    pub enabled: bool,
    pub remark: String,
    pub allowed_user_ids: Vec<String>,
}
impl From<AdminProductRequestDto> for AdminProductInput {
    fn from(v: AdminProductRequestDto) -> Self {
        Self {
            membership_tier: v.membership_tier,
            name: v.name,
            product_id: v.product_id,
            price_cents: v.price_cents,
            membership_duration_months: v.membership_duration_months,
            enabled: v.enabled,
            remark: v.remark,
            allowed_user_ids: v.allowed_user_ids,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct AdminProductDto {
    pub product_key: String,
    pub membership_tier: String,
    pub name: String,
    pub product_id: String,
    pub price_cents: i32,
    pub membership_duration_months: i32,
    pub enabled: bool,
    pub remark: String,
    pub allowed_user_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl From<AdminProduct> for AdminProductDto {
    fn from(v: AdminProduct) -> Self {
        Self {
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
#[derive(Debug, Serialize)]
pub struct AdminProductPageDto {
    pub items: Vec<AdminProductDto>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}
impl From<AdminProductPage> for AdminProductPageDto {
    fn from(v: AdminProductPage) -> Self {
        Self {
            items: v.items.into_iter().map(Into::into).collect(),
            total: v.total,
            page: v.page,
            page_size: v.page_size,
        }
    }
}
#[derive(Debug, Deserialize, Default)]
pub struct AdminProductListQuery {
    pub query: Option<String>,
    pub membership_tier: Option<String>,
    pub enabled: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
impl TryFrom<AdminProductListQuery> for AdminProductQuery {
    type Error = AdminProductError;
    fn try_from(v: AdminProductListQuery) -> Result<Self, Self::Error> {
        let enabled = match v.enabled.as_deref().map(str::trim) {
            None | Some("") => None,
            Some("true") => Some(true),
            Some("false") => Some(false),
            _ => {
                return Err(AdminProductError::InvalidInput(
                    "enabled must be true or false".into(),
                ));
            }
        };
        Ok(Self {
            query: v.query,
            membership_tier: v.membership_tier,
            enabled,
            page: v.page.unwrap_or(1),
            page_size: v.page_size.unwrap_or(20),
        })
    }
}

fn default_membership_duration_months() -> i32 {
    12
}
