use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Paid,
    Failed,
    Refunded,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Paid => "paid",
            Self::Failed => "failed",
            Self::Refunded => "refunded",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "paid" => Self::Paid,
            "failed" => Self::Failed,
            "refunded" => Self::Refunded,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaymentOrder {
    pub id: Uuid,
    pub order_no: String,
    pub user_id: Uuid,
    pub amount_cents: i32,
    pub status: OrderStatus,
    pub prepay_id: Option<String>,
    pub transaction_id: Option<String>,
    pub product_type: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewPaymentOrder {
    pub order_no: String,
    pub user_id: Uuid,
    pub amount_cents: i32,
    pub product_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipProduct {
    pub price_cents: i32,
    pub title: String,
    pub subtitle: String,
    pub description: String,
    #[serde(default = "default_membership_product_options")]
    pub products: Vec<MembershipProductOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipProductOption {
    pub target_tier: String,
    pub price_cents: i32,
    pub title: String,
    pub subtitle: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipCheckoutPrice {
    pub original_price_cents: i32,
    pub pay_price_cents: i32,
    pub upgrade_fee_cents: i32,
}

#[derive(Debug, Clone)]
pub struct WxPayParams {
    pub time_stamp: String,
    pub nonce_str: String,
    pub package: String,
    pub sign_type: String,
    pub pay_sign: String,
}

pub fn default_membership_product_options() -> Vec<MembershipProductOption> {
    vec![
        MembershipProductOption {
            target_tier: "V6".to_string(),
            price_cents: 3900,
            title: "V6 回流速览会员".to_string(),
            subtitle: "查看 30 分钟内".to_string(),
            description: "购买后立即升级为 V6 会员，可查看最近 30 分钟内的回流区域。".to_string(),
        },
        MembershipProductOption {
            target_tier: "V7".to_string(),
            price_cents: 5900,
            title: "V7 进阶回流会员".to_string(),
            subtitle: "查看 10 分钟内".to_string(),
            description: "购买后立即升级为 V7 会员，可查看最近 10 分钟内的回流区域。".to_string(),
        },
        MembershipProductOption {
            target_tier: "V8".to_string(),
            price_cents: 7900,
            title: "V8 高阶回流会员".to_string(),
            subtitle: "查看 3 分钟内".to_string(),
            description: "购买后立即升级为 V8 会员，可查看最近 3 分钟内的回流区域。".to_string(),
        },
        MembershipProductOption {
            target_tier: "V9".to_string(),
            price_cents: 9900,
            title: "V9 旗舰回流会员".to_string(),
            subtitle: "同 V8 速览权限".to_string(),
            description: "购买后立即升级为 V9 会员，最近回流速览权限与 V8 一致。".to_string(),
        },
    ]
}

impl MembershipProduct {
    pub fn normalized(mut self) -> Self {
        if self.products.is_empty() {
            self.products = default_membership_product_options();
        }

        self
    }

    pub fn find_option(&self, target_tier: &str) -> Option<&MembershipProductOption> {
        let normalized_tier = target_tier.trim().to_uppercase();
        self.products
            .iter()
            .find(|product| product.target_tier.trim().to_uppercase() == normalized_tier)
    }
}

pub fn membership_product_type_for_tier(target_tier: &str) -> String {
    format!("membership:{}", target_tier.trim().to_uppercase())
}

pub fn membership_tier_from_product_type(product_type: &str) -> String {
    product_type
        .trim()
        .strip_prefix("membership:")
        .map(|tier| tier.trim().to_uppercase())
        .filter(|tier| matches!(tier.as_str(), "V6" | "V7" | "V8" | "V9"))
        .unwrap_or_else(|| "V9".to_string())
}

pub fn calculate_membership_checkout_price(
    products: &[MembershipProductOption],
    current_tier: &str,
    target_tier: &str,
) -> anyhow::Result<MembershipCheckoutPrice> {
    let current_tier = current_tier.trim().to_uppercase();
    let target_tier = target_tier.trim().to_uppercase();

    if crate::auth::domain::membership::membership_tier_rank(&target_tier)
        <= crate::auth::domain::membership::membership_tier_rank(&current_tier)
    {
        anyhow::bail!("请选择高于当前等级的会员档位");
    }

    let target_product = products
        .iter()
        .find(|product| {
            product
                .target_tier
                .trim()
                .eq_ignore_ascii_case(&target_tier)
        })
        .ok_or_else(|| anyhow::anyhow!("请选择有效的会员档位"))?;

    let original_price_cents = target_product.price_cents;
    let current_product = match current_tier.as_str() {
        "V6" | "V7" | "V8" => products.iter().find(|product| {
            product
                .target_tier
                .trim()
                .eq_ignore_ascii_case(&current_tier)
        }),
        _ => None,
    };

    let Some(current_product) = current_product else {
        return Ok(MembershipCheckoutPrice {
            original_price_cents,
            pay_price_cents: original_price_cents,
            upgrade_fee_cents: 0,
        });
    };

    let upgrade_fee_cents = 1500;
    Ok(MembershipCheckoutPrice {
        original_price_cents,
        pay_price_cents: original_price_cents - current_product.price_cents + upgrade_fee_cents,
        upgrade_fee_cents,
    })
}

#[cfg(test)]
mod tests {
    use super::{calculate_membership_checkout_price, default_membership_product_options};

    #[test]
    fn calculate_membership_checkout_price_charges_difference_plus_upgrade_fee() {
        let products = default_membership_product_options();

        for (current_tier, target_tier, original_price_cents, pay_price_cents) in [
            ("V6", "V7", 5900, 3500),
            ("V6", "V8", 7900, 5500),
            ("V7", "V8", 7900, 3500),
            ("V8", "V9", 9900, 3500),
        ] {
            let price = calculate_membership_checkout_price(&products, current_tier, target_tier)
                .expect("upgrade price");

            assert_eq!(price.original_price_cents, original_price_cents);
            assert_eq!(price.pay_price_cents, pay_price_cents);
            assert_eq!(price.upgrade_fee_cents, 1500);
        }
    }

    #[test]
    fn calculate_membership_checkout_price_uses_original_price_outside_paid_upgrade_range() {
        let products = default_membership_product_options();

        let price =
            calculate_membership_checkout_price(&products, "V3", "V8").expect("V3 to V8 price");

        assert_eq!(price.original_price_cents, 7900);
        assert_eq!(price.pay_price_cents, 7900);
        assert_eq!(price.upgrade_fee_cents, 0);
    }

    #[test]
    fn calculate_membership_checkout_price_rejects_same_tier() {
        let products = default_membership_product_options();

        let error = calculate_membership_checkout_price(&products, "V8", "V8")
            .expect_err("same tier should be rejected");

        assert!(error.to_string().contains("高于当前等级"));
    }
}
