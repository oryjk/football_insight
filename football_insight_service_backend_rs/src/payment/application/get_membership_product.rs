use std::sync::Arc;

use crate::payment::domain::order::MembershipProduct;

pub struct GetMembershipProductUseCase {
    system_config_port: Arc<dyn crate::system_config::ports::system_config_port::SystemConfigPort>,
    products: Arc<dyn crate::payment::ports::virtual_payment_port::VirtualProductRepository>,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::GetMembershipProductUseCase;
    use crate::system_config::{
        domain::{ai_chat_config::AiChatSystemConfig, public_system_config::PublicSystemConfig},
        ports::system_config_port::SystemConfigPort,
    };

    struct FakeSystemConfigPort {
        value: Mutex<Option<String>>,
    }

    struct EmptyProductRepository;
    #[async_trait]
    impl crate::payment::ports::virtual_payment_port::VirtualProductRepository
        for EmptyProductRepository
    {
        async fn find_enabled(
            &self,
            _: &str,
            _: uuid::Uuid,
        ) -> Result<
            Option<crate::payment::domain::virtual_payment::VirtualProduct>,
            crate::payment::domain::virtual_payment::VirtualPaymentError,
        > {
            Ok(None)
        }
        async fn has_enabled(
            &self,
        ) -> Result<bool, crate::payment::domain::virtual_payment::VirtualPaymentError> {
            Ok(false)
        }
        async fn private_membership_products(
            &self,
            _: uuid::Uuid,
        ) -> Result<
            Vec<crate::payment::domain::virtual_payment::PrivateMembershipProduct>,
            crate::payment::domain::virtual_payment::VirtualPaymentError,
        > {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl SystemConfigPort for FakeSystemConfigPort {
        async fn get_public_config(&self) -> anyhow::Result<PublicSystemConfig> {
            unreachable!()
        }

        async fn get_ai_chat_config(&self) -> anyhow::Result<AiChatSystemConfig> {
            unreachable!()
        }

        async fn get_config_value(&self, _config_key: &str) -> anyhow::Result<Option<String>> {
            Ok(self.value.lock().expect("value").clone())
        }
    }

    #[tokio::test]
    async fn legacy_padded_v3_option_is_not_visible_in_public_catalog() {
        let use_case = GetMembershipProductUseCase::new(
            Arc::new(FakeSystemConfigPort { value: Mutex::new(Some(serde_json::json!({
                "price_cents":1,"title":"member","subtitle":"","description":"",
                "products":[{"target_tier":" v3 ","price_cents":1,"title":"private","subtitle":"","description":""}]
            }).to_string())) }),
            Arc::new(EmptyProductRepository),
        );
        assert!(
            use_case
                .execute()
                .await
                .unwrap()
                .find_option("V3")
                .is_none()
        );
    }

    #[tokio::test]
    async fn execute_returns_v6_to_v9_membership_products_by_default() {
        let use_case = GetMembershipProductUseCase::new(
            std::sync::Arc::new(FakeSystemConfigPort {
                value: Mutex::new(None),
            }),
            Arc::new(EmptyProductRepository),
        );

        let catalog = use_case.execute().await.expect("catalog");

        assert_eq!(
            catalog
                .products
                .iter()
                .map(|product| (product.target_tier.as_str(), product.price_cents))
                .collect::<Vec<_>>(),
            vec![("V6", 3900), ("V7", 5900), ("V8", 7900), ("V9", 9900)]
        );
    }
}

impl GetMembershipProductUseCase {
    pub fn new(
        system_config_port: Arc<
            dyn crate::system_config::ports::system_config_port::SystemConfigPort,
        >,
        products: Arc<dyn crate::payment::ports::virtual_payment_port::VirtualProductRepository>,
    ) -> Self {
        Self {
            system_config_port,
            products,
        }
    }

    pub async fn execute(&self) -> anyhow::Result<MembershipProduct> {
        self.execute_for_user(None).await
    }

    pub async fn execute_for_user(
        &self,
        user_id: Option<uuid::Uuid>,
    ) -> anyhow::Result<MembershipProduct> {
        let config_value = self
            .system_config_port
            .get_config_value("membership_product")
            .await?;

        let product: MembershipProduct = match config_value {
            Some(value) => serde_json::from_str(&value).unwrap_or_else(|_| Self::default_product()),
            None => Self::default_product(),
        };

        let mut product = product.normalized();
        // V3 is never a public paid option, even if an old system-config catalog contains it.
        product
            .products
            .retain(|p| !p.target_tier.trim().eq_ignore_ascii_case("V3"));
        if let Some(user_id) = user_id {
            for private in self.products.private_membership_products(user_id).await? {
                let tier = crate::payment::domain::order::membership_tier_from_product_type(
                    &private.product_type,
                );
                if product.find_option(&tier).is_some() {
                    continue;
                }
                product
                    .products
                    .push(crate::payment::domain::order::MembershipProductOption {
                        target_tier: tier.clone(),
                        price_cents: private.price_cents,
                        title: private.name,
                        subtitle: "仅限指定账号".into(),
                        description: format!("支付确认后开通 {tier}，权益按现有等级规则执行。"),
                        membership_duration_months: Some(private.membership_duration_months),
                    });
            }
        }
        Ok(product)
    }

    pub async fn with_duration_for_tier(
        &self,
        mut product: MembershipProduct,
        current_tier: Option<&str>,
        user_id: Option<uuid::Uuid>,
    ) -> anyhow::Result<MembershipProduct> {
        let options = product.products.clone();
        for option in &mut product.products {
            let price = current_tier
                .and_then(|tier| {
                    crate::payment::domain::order::calculate_membership_checkout_price(
                        &options,
                        tier,
                        &option.target_tier,
                    )
                    .ok()
                })
                .map(|price| price.pay_price_cents)
                .unwrap_or(option.price_cents);
            option.membership_duration_months = None;
            let Ok(key) = crate::payment::domain::virtual_payment::product_key(
                &crate::payment::domain::order::membership_product_type_for_tier(
                    &option.target_tier,
                ),
                price,
            ) else {
                continue;
            };
            option.membership_duration_months = self
                .products
                .find_enabled(&key, user_id.unwrap_or_default())
                .await?
                .map(|product| product.membership_duration_months);
        }
        Ok(product)
    }

    fn default_product() -> MembershipProduct {
        MembershipProduct {
            price_cents: 9900,
            title: "足球洞察会员".to_string(),
            subtitle: "解锁全部高级功能".to_string(),
            description: "购买后立即升级为 V9 会员，享受全部特权。".to_string(),
            products: crate::payment::domain::order::default_membership_product_options(),
        }
    }
}
