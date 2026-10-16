use chrono::{DateTime, Duration, FixedOffset, Months, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// The single catalog tier rule. Callers normalize case/whitespace explicitly when appropriate.
pub fn is_valid_membership_tier(tier: &str) -> bool {
    matches!(tier.as_bytes(), [b'V', b'3'..=b'9'])
}

#[derive(Debug, thiserror::Error)]
pub enum ProductSpecError {
    #[error("{0}")]
    Invalid(String),
    #[error("请更新小程序后购买")]
    CapabilityRequired,
    #[error("当前会员等级不可购买该商品")]
    Ineligible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "rule", rename_all = "snake_case", deny_unknown_fields)]
pub enum MembershipExpiry {
    Months { months: i32 },
    EndOfNextDay { tz: String },
}
impl MembershipExpiry {
    pub fn validate(&self) -> Result<(), ProductSpecError> {
        match self {
            Self::Months { months } if (1..=1200).contains(months) => Ok(()),
            Self::EndOfNextDay { tz } if tz == "Asia/Shanghai" => Ok(()),
            _ => Err(ProductSpecError::Invalid(
                "invalid membership expiry".into(),
            )),
        }
    }
    pub fn expires_at(&self, now: DateTime<Utc>) -> Result<DateTime<Utc>, ProductSpecError> {
        self.validate()?;
        let result = match self {
            Self::Months { months } => now.checked_add_months(Months::new(*months as u32)),
            Self::EndOfNextDay { .. } => {
                let tz = FixedOffset::east_opt(8 * 3600).expect("Beijing offset");
                now.with_timezone(&tz)
                    .date_naive()
                    .checked_add_signed(Duration::days(2))
                    .and_then(|day| day.and_hms_opt(0, 0, 0))
                    .and_then(|time| tz.from_local_datetime(&time).single())
                    .and_then(|time| time.checked_sub_signed(Duration::seconds(1)))
                    .map(|v| v.with_timezone(&Utc))
            }
        };
        result.ok_or_else(|| ProductSpecError::Invalid("membership expiry overflow".into()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipGrant {
    pub tier: String,
    pub expiry: MembershipExpiry,
}
impl MembershipGrant {
    pub fn validate(&self) -> Result<(), ProductSpecError> {
        if !is_valid_membership_tier(&self.tier) {
            return Err(ProductSpecError::Invalid("invalid membership tier".into()));
        }
        self.expiry.validate()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantSnapshot {
    pub kind: String,
    pub grant_spec: Value,
    pub catalog_product_id: Option<Uuid>,
}
impl GrantSnapshot {
    pub fn from_spec(spec: &ProductSpec) -> Result<Self, ProductSpecError> {
        ProductKindRegistry::default().validate(spec)?;
        Ok(Self {
            kind: spec.kind.clone(),
            grant_spec: spec.grant_spec.clone(),
            catalog_product_id: Some(spec.catalog_product_id),
        })
    }
    pub fn membership(&self) -> Result<MembershipGrant, ProductSpecError> {
        if self.kind != "membership" {
            return Err(ProductSpecError::Invalid("unknown product kind".into()));
        }
        let grant: MembershipGrant = serde_json::from_value(self.grant_spec.clone())
            .map_err(|_| ProductSpecError::Invalid("invalid membership grant".into()))?;
        grant.validate()?;
        Ok(grant)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSpec {
    pub catalog_product_id: Uuid,
    pub product_key: String,
    pub kind: String,
    pub price_cents: i32,
    pub membership_duration_months: i32,
    pub grant_spec: Value,
    pub pricing_rule: Value,
    pub eligibility: Value,
    pub requires_capability: Option<String>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    pub group_key: Option<String>,
}
impl ProductSpec {
    pub fn is_table_offer(&self) -> bool {
        self.title.is_some() || self.requires_capability.is_some() || self.group_key.is_some()
    }
    pub fn is_pass(&self) -> bool {
        self.grant_spec
            .pointer("/expiry/rule")
            .and_then(Value::as_str)
            == Some("end_of_next_day")
    }
    pub fn capability_allowed(&self, capabilities: &[String]) -> bool {
        self.requires_capability
            .as_ref()
            .is_none_or(|required| capabilities.contains(required))
    }
}

#[derive(Debug, Clone)]
pub struct UserContext {
    pub current_tier: String,
    pub capabilities: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Eligibility {
    max_current_tier: Option<String>,
    min_current_tier: Option<String>,
}

/// Pure kind rules; transactional fulfillment stays on the settlement port/adapters.
pub trait ProductKind: Send + Sync {
    fn name(&self) -> &'static str;
    fn validate(&self, spec: &ProductSpec) -> Result<(), ProductSpecError>;
    fn eligible(&self, spec: &ProductSpec, user: &UserContext) -> bool;
    fn resolve_price(
        &self,
        spec: &ProductSpec,
        user: &UserContext,
    ) -> Result<i32, ProductSpecError>;
}
struct MembershipKind;
impl ProductKind for MembershipKind {
    fn name(&self) -> &'static str {
        "membership"
    }
    fn validate(&self, spec: &ProductSpec) -> Result<(), ProductSpecError> {
        let invalid = || ProductSpecError::Invalid("invalid membership product spec".into());
        let grant = GrantSnapshot {
            kind: spec.kind.clone(),
            grant_spec: spec.grant_spec.clone(),
            catalog_product_id: Some(spec.catalog_product_id),
        }
        .membership()?;
        if spec.product_key != format!("membership:{}@{}", grant.tier, spec.price_cents)
            || spec.price_cents <= 0
            || spec.pricing_rule != serde_json::json!({"type":"fixed"})
        {
            return Err(invalid());
        }
        let e: Eligibility =
            serde_json::from_value(spec.eligibility.clone()).map_err(|_| invalid())?;
        for tier in [&e.max_current_tier, &e.min_current_tier]
            .into_iter()
            .flatten()
        {
            if !is_valid_membership_tier(tier) {
                return Err(invalid());
            }
        }
        if matches!(grant.expiry, MembershipExpiry::EndOfNextDay { .. })
            && (e.max_current_tier.as_deref() != Some("V3")
                || spec.requires_capability.as_deref() != Some("catalog-v2"))
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn eligible(&self, spec: &ProductSpec, user: &UserContext) -> bool {
        if self.validate(spec).is_err() || !spec.capability_allowed(&user.capabilities) {
            return false;
        }
        let Ok(e) = serde_json::from_value::<Eligibility>(spec.eligibility.clone()) else {
            return false;
        };
        let rank = crate::auth::domain::membership::membership_tier_rank(&user.current_tier);
        e.max_current_tier
            .as_ref()
            .is_none_or(|v| rank <= crate::auth::domain::membership::membership_tier_rank(v))
            && e.min_current_tier
                .as_ref()
                .is_none_or(|v| rank >= crate::auth::domain::membership::membership_tier_rank(v))
    }
    fn resolve_price(
        &self,
        spec: &ProductSpec,
        user: &UserContext,
    ) -> Result<i32, ProductSpecError> {
        self.validate(spec)?;
        if !spec.capability_allowed(&user.capabilities) {
            return Err(ProductSpecError::CapabilityRequired);
        }
        if !self.eligible(spec, user) {
            return Err(ProductSpecError::Ineligible);
        }
        Ok(spec.price_cents)
    }
}

pub struct ProductKindRegistry {
    kinds: HashMap<&'static str, Arc<dyn ProductKind>>,
}
impl Default for ProductKindRegistry {
    fn default() -> Self {
        let membership = Arc::new(MembershipKind) as Arc<dyn ProductKind>;
        Self {
            kinds: HashMap::from([(membership.name(), membership)]),
        }
    }
}
impl ProductKindRegistry {
    fn get(&self, kind: &str) -> Result<&dyn ProductKind, ProductSpecError> {
        self.kinds
            .get(kind)
            .map(AsRef::as_ref)
            .ok_or_else(|| ProductSpecError::Invalid("unknown product kind".into()))
    }
    pub fn validate(&self, spec: &ProductSpec) -> Result<(), ProductSpecError> {
        self.get(&spec.kind)?.validate(spec)
    }
    pub fn eligible(&self, spec: &ProductSpec, user: &UserContext) -> bool {
        self.get(&spec.kind)
            .is_ok_and(|kind| kind.eligible(spec, user))
    }
    pub fn resolve_price(
        &self,
        spec: &ProductSpec,
        user: &UserContext,
    ) -> Result<i32, ProductSpecError> {
        self.get(&spec.kind)?.resolve_price(spec, user)
    }
}
