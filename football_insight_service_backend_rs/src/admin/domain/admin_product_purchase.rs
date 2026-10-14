use chrono::{DateTime, Utc};
use uuid::Uuid;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PurchaseExpirySource {
    Snapshot,
    LegacyYear,
    Unrecorded,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProductPurchaseSummary {
    pub purchased_user_count: i64,
    pub purchased_order_count: i64,
    pub refunded_order_count: i64,
    pub total_amount_cents: i64,
}
#[derive(Debug, Clone)]
pub struct ProductPurchase {
    pub order_no: String,
    pub user_id: Uuid,
    pub user_nickname: String,
    pub user_avatar_url: Option<String>,
    pub paid_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub expiry_source: PurchaseExpirySource,
    pub amount_cents: i32,
    pub status: String,
}
#[derive(Debug, Clone)]
pub struct ProductMember {
    pub user_id: Uuid,
    pub user_nickname: String,
    pub user_avatar_url: Option<String>,
    pub membership_origin: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub expired: bool,
}
#[derive(Debug, Clone)]
pub struct ProductMemberPage {
    pub catalog_product_id: Uuid,
    pub membership_tier: String,
    pub items: Vec<ProductMember>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}
#[derive(Debug, Clone)]
pub struct ProductPurchasePage {
    pub catalog_product_id: Uuid,
    pub summary: ProductPurchaseSummary,
    pub items: Vec<ProductPurchase>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}
pub fn resolve_purchase_expiry(
    paid_at: DateTime<Utc>,
    has_virtual: bool,
    snapshot: Option<Option<DateTime<Utc>>>,
) -> (Option<DateTime<Utc>>, PurchaseExpirySource) {
    match snapshot {
        Some(expiry) => (expiry, PurchaseExpirySource::Snapshot),
        None if has_virtual => (None, PurchaseExpirySource::Unrecorded),
        None => (
            paid_at.checked_add_months(chrono::Months::new(12)),
            PurchaseExpirySource::LegacyYear,
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn time(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }
    #[test]
    fn expiry_uses_recorded_snapshot_and_never_current_user_membership() {
        let paid = time("2024-02-29T10:00:00Z");
        let granted = time("2024-03-29T10:00:00Z");
        assert_eq!(
            resolve_purchase_expiry(paid, true, Some(Some(granted))),
            (Some(granted), PurchaseExpirySource::Snapshot)
        );
        assert_eq!(
            resolve_purchase_expiry(paid, true, Some(None)),
            (None, PurchaseExpirySource::Snapshot)
        );
        assert_eq!(
            resolve_purchase_expiry(paid, true, None),
            (None, PurchaseExpirySource::Unrecorded)
        );
        assert_eq!(
            resolve_purchase_expiry(paid, false, None),
            (
                Some(time("2025-02-28T10:00:00Z")),
                PurchaseExpirySource::LegacyYear
            )
        );
    }
}
