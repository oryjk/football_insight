use serde::{Deserialize, Serialize};

use crate::payment::domain::order::{
    MembershipProduct, MembershipProductOption, WxPayParams, calculate_membership_checkout_price,
};

#[derive(Debug, Serialize)]
pub struct MembershipProductResponse {
    pub price_cents: i32,
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub products: Vec<MembershipProductOptionResponse>,
}

#[derive(Debug, Serialize)]
pub struct MembershipProductOptionResponse {
    pub target_tier: String,
    pub price_cents: i32,
    pub original_price_cents: i32,
    pub pay_price_cents: i32,
    pub upgrade_fee_cents: i32,
    pub title: String,
    pub subtitle: String,
    pub description: String,
}

impl MembershipProductResponse {
    pub fn from_product_for_tier(value: MembershipProduct, current_tier: Option<&str>) -> Self {
        let products = value.products.clone();

        Self {
            price_cents: value.price_cents,
            title: value.title,
            subtitle: value.subtitle,
            description: value.description,
            products: value
                .products
                .into_iter()
                .map(|product| {
                    MembershipProductOptionResponse::from_product_for_tier(
                        product,
                        &products,
                        current_tier,
                    )
                })
                .collect(),
        }
    }
}

impl MembershipProductOptionResponse {
    fn from_product_for_tier(
        value: MembershipProductOption,
        products: &[MembershipProductOption],
        current_tier: Option<&str>,
    ) -> Self {
        let original_price_cents = value.price_cents;
        let price = current_tier.and_then(|tier| {
            calculate_membership_checkout_price(products, tier, &value.target_tier).ok()
        });

        Self {
            target_tier: value.target_tier,
            price_cents: value.price_cents,
            original_price_cents,
            pay_price_cents: price
                .as_ref()
                .map(|price| price.pay_price_cents)
                .unwrap_or(original_price_cents),
            upgrade_fee_cents: price
                .as_ref()
                .map(|price| price.upgrade_fee_cents)
                .unwrap_or(0),
            title: value.title,
            subtitle: value.subtitle,
            description: value.description,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateMembershipOrderRequest {
    pub target_tier: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    pub order_no: String,
    pub params: WxPayParamsDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WxPayParamsDto {
    pub time_stamp: String,
    pub nonce_str: String,
    pub package: String,
    pub sign_type: String,
    pub pay_sign: String,
}

impl From<WxPayParams> for WxPayParamsDto {
    fn from(value: WxPayParams) -> Self {
        Self {
            time_stamp: value.time_stamp,
            nonce_str: value.nonce_str,
            package: value.package,
            sign_type: value.sign_type,
            pay_sign: value.pay_sign,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OrderStatusResponse {
    pub order_no: String,
    pub status: String,
    pub amount_cents: i32,
    pub paid_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WechatNotifyBody {
    pub xml: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{MembershipProductResponse, WxPayParamsDto};
    use crate::payment::domain::order::{
        MembershipProduct, WxPayParams, default_membership_product_options,
    };

    #[test]
    fn wx_pay_params_dto_serializes_camel_case_fields() {
        let dto = WxPayParamsDto::from(WxPayParams {
            time_stamp: "1710000000".to_string(),
            nonce_str: "nonce".to_string(),
            package: "prepay_id=wx123".to_string(),
            sign_type: "MD5".to_string(),
            pay_sign: "SIGN".to_string(),
        });

        let value = serde_json::to_value(dto).expect("serialize dto");

        assert_eq!(
            value,
            json!({
                "timeStamp": "1710000000",
                "nonceStr": "nonce",
                "package": "prepay_id=wx123",
                "signType": "MD5",
                "paySign": "SIGN"
            })
        );
    }

    #[test]
    fn membership_product_response_includes_computed_upgrade_prices() {
        let dto = MembershipProductResponse::from_product_for_tier(
            MembershipProduct {
                price_cents: 9900,
                title: "会员".to_string(),
                subtitle: "sub".to_string(),
                description: "desc".to_string(),
                products: default_membership_product_options(),
            },
            Some("V6"),
        );

        let v7 = dto
            .products
            .iter()
            .find(|product| product.target_tier == "V7")
            .expect("V7 product");

        assert_eq!(v7.original_price_cents, 5900);
        assert_eq!(v7.pay_price_cents, 3500);
        assert_eq!(v7.upgrade_fee_cents, 1500);
    }
}
