use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdminProductError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("product not found")]
    NotFound,
    #[error("product key already exists")]
    Conflict,
    #[error("product operation failed")]
    Internal,
}

#[derive(Debug, Clone)]
pub struct AdminProductInput {
    pub membership_tier: String,
    pub name: String,
    pub product_id: String,
    pub price_cents: i32,
    pub membership_duration_months: i32,
    pub enabled: bool,
    pub remark: String,
    pub allowed_user_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidatedAdminProduct {
    pub product_key: String,
    pub membership_tier: String,
    pub name: String,
    pub product_id: String,
    pub price_cents: i32,
    pub membership_duration_months: i32,
    pub enabled: bool,
    pub remark: String,
    pub allowed_user_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminProduct {
    pub catalog_product_id: Uuid,
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

#[derive(Debug, Clone)]
pub struct AdminProductQuery {
    pub query: Option<String>,
    pub membership_tier: Option<String>,
    pub enabled: Option<bool>,
    pub page: u32,
    pub page_size: u32,
}

impl Default for AdminProductQuery {
    fn default() -> Self {
        Self {
            query: None,
            membership_tier: None,
            enabled: None,
            page: 1,
            page_size: 20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdminProductPage {
    pub items: Vec<AdminProduct>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
}

impl AdminProductInput {
    pub fn validate(self) -> Result<ValidatedAdminProduct, AdminProductError> {
        let membership_tier = validate_tier(&self.membership_tier)?.to_owned();
        let name = bounded_text(&self.name, "name", 200, true)?;
        let product_id = bounded_text(&self.product_id, "product_id", 200, true)?;
        if product_id.chars().any(char::is_whitespace) {
            return Err(invalid("product_id must not contain whitespace"));
        }
        let remark = bounded_text(&self.remark, "remark", 4000, false)?;
        if !(1..=1200).contains(&self.membership_duration_months) {
            return Err(invalid(
                "membership_duration_months must be an integer between 1 and 1200",
            ));
        }
        if self.price_cents <= 0 {
            return Err(invalid("price_cents must be a positive integer"));
        }
        if self.allowed_user_ids.len() > 100 {
            return Err(invalid(
                "allowed_user_ids must contain at most 100 accounts",
            ));
        }
        let mut allowed_user_ids = Vec::new();
        for value in self.allowed_user_ids {
            let value = value.trim();
            if value.len() != 36 {
                return Err(invalid("allowed_user_ids must contain valid UUIDs"));
            }
            let id = Uuid::parse_str(value)
                .map_err(|_| invalid("allowed_user_ids must contain valid UUIDs"))?;
            if id.is_nil() {
                return Err(invalid("allowed_user_ids must not contain the nil UUID"));
            }
            if !allowed_user_ids.contains(&id) {
                allowed_user_ids.push(id);
            }
        }
        if membership_tier == "V3" && allowed_user_ids.is_empty() {
            return Err(invalid("V3 requires at least one allowed_user_ids account"));
        }
        Ok(ValidatedAdminProduct {
            product_key: format!("membership:{membership_tier}@{}", self.price_cents),
            membership_tier,
            name,
            product_id,
            price_cents: self.price_cents,
            membership_duration_months: self.membership_duration_months,
            enabled: self.enabled,
            remark,
            allowed_user_ids,
        })
    }
}

impl AdminProductQuery {
    pub fn validate(mut self) -> Result<Self, AdminProductError> {
        if self.page == 0 || !(1..=100).contains(&self.page_size) {
            return Err(invalid(
                "page must be positive and page_size must be between 1 and 100",
            ));
        }
        self.query = self
            .query
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        if self
            .query
            .as_ref()
            .is_some_and(|value| value.chars().count() > 200)
        {
            return Err(invalid("query must contain at most 200 characters"));
        }
        self.membership_tier = self
            .membership_tier
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty());
        if let Some(tier) = &self.membership_tier {
            validate_tier(tier)?;
        }
        Ok(self)
    }
}

pub fn validate_product_key(key: &str) -> Result<(), AdminProductError> {
    let (tier, amount) = key
        .strip_prefix("membership:")
        .and_then(|value| value.split_once('@'))
        .ok_or_else(|| invalid("invalid membership product key"))?;
    if validate_tier(tier)? != tier {
        return Err(invalid("invalid membership product key"));
    }
    let price = amount
        .parse::<i32>()
        .map_err(|_| invalid("invalid membership product key"))?;
    if price <= 0 || price.to_string() != amount {
        return Err(invalid("invalid membership product key"));
    }
    Ok(())
}

fn invalid(message: &str) -> AdminProductError {
    AdminProductError::InvalidInput(message.into())
}
fn validate_tier(value: &str) -> Result<&str, AdminProductError> {
    match value.trim() {
        tier @ ("V3" | "V6" | "V7" | "V8" | "V9") => Ok(tier),
        _ => Err(invalid("membership_tier must be V3, V6, V7, V8 or V9")),
    }
}
fn bounded_text(
    value: &str,
    field: &str,
    limit: usize,
    required: bool,
) -> Result<String, AdminProductError> {
    let value = value.trim();
    if required && value.is_empty() {
        return Err(invalid(&format!("{field} is required")));
    }
    if value.chars().count() > limit
        || value
            .chars()
            .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
    {
        return Err(invalid(&format!(
            "{field} is too long or contains invalid characters"
        )));
    }
    Ok(value.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> AdminProductInput {
        AdminProductInput {
            membership_tier: " V6 ".into(),
            name: " Annual ".into(),
            product_id: " wx-id ".into(),
            price_cents: 3900,
            membership_duration_months: 12,
            enabled: false,
            remark: " note ".into(),
            allowed_user_ids: vec![],
        }
    }
    #[test]
    fn normalizes_and_generates_membership_key() {
        let value = input().validate().unwrap();
        assert_eq!(value.product_key, "membership:V6@3900");
        assert_eq!(value.name, "Annual");
        assert_eq!(value.product_id, "wx-id");
        assert_eq!(value.remark, "note");
    }
    #[test]
    fn rejects_unsupported_tiers_and_nonpositive_prices() {
        for tier in ["V1", "V4", "v6", "match_id_unlock"] {
            let mut v = input();
            v.membership_tier = tier.into();
            assert!(v.validate().is_err());
        }
        for price in [0, -1] {
            let mut v = input();
            v.price_cents = price;
            assert!(v.validate().is_err());
        }
    }
    #[test]
    fn requires_v3_accounts_and_valid_uuid_and_deduplicates() {
        let mut v = input();
        v.membership_tier = "V3".into();
        assert!(v.clone().validate().is_err());
        v.allowed_user_ids = vec!["invalid".into()];
        assert!(v.clone().validate().is_err());
        v.allowed_user_ids = vec!["00000000-0000-0000-0000-000000000000".into()];
        assert!(v.clone().validate().is_err());
        let id = "11111111-1111-4111-8111-111111111111";
        v.allowed_user_ids = vec![id.into(), format!(" {id} ")];
        assert_eq!(v.validate().unwrap().allowed_user_ids.len(), 1);
    }
    #[test]
    fn rejects_blank_fields_and_nonmembership_or_noncanonical_keys() {
        for field in ["name", "product_id"] {
            let mut v = input();
            if field == "name" {
                v.name = " ".into();
            } else {
                v.product_id = " ".into();
            }
            assert!(v.validate().is_err());
        }
        for key in [
            "match_id_unlock@100",
            "membership:V6@03900",
            "membership:V6@0",
            "membership:V4@100",
            "membership:V6@999999999999",
        ] {
            assert!(validate_product_key(key).is_err());
        }
        assert!(validate_product_key("membership:V3@100").is_ok());
    }
    #[test]
    fn membership_duration_accepts_boundaries_and_rejects_out_of_range() {
        for months in [1, 12, 1200] {
            let mut value = input();
            value.membership_duration_months = months;
            assert_eq!(value.validate().unwrap().membership_duration_months, months);
        }
        for months in [0, -1, 1201] {
            let mut value = input();
            value.membership_duration_months = months;
            assert!(matches!(
                value.validate(),
                Err(AdminProductError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn pagination_is_bounded_and_search_is_normalized() {
        let query = AdminProductQuery {
            query: Some("  annual  ".into()),
            ..Default::default()
        }
        .validate()
        .unwrap();
        assert_eq!(query.query, Some("annual".into()));
        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 20);
        for (page, size) in [(0, 20), (1, 0), (1, 101)] {
            assert!(
                AdminProductQuery {
                    page,
                    page_size: size,
                    ..Default::default()
                }
                .validate()
                .is_err()
            );
        }
    }
}
