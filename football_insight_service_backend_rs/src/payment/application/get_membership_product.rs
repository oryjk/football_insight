use std::sync::Arc;

use crate::payment::domain::order::MembershipProduct;

pub struct GetMembershipProductUseCase {
    registry: Arc<crate::payment::domain::product_kind::ProductKindRegistry>,
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
            registry: Arc::new(
                crate::payment::domain::product_kind::ProductKindRegistry::default(),
            ),
            system_config_port,
            products,
        }
    }

    pub fn with_kind_registry(
        mut self,
        registry: Arc<crate::payment::domain::product_kind::ProductKindRegistry>,
    ) -> Self {
        self.registry = registry;
        self
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
            let mut private_products = self.products.private_membership_products(user_id).await?;
            // The API selects one offer per tier. Prefer the cheapest eligible direct offer,
            // while retaining system-config prices as the credit basis for other upgrades.
            private_products.sort_by(|a, b| {
                a.price_cents
                    .cmp(&b.price_cents)
                    .then_with(|| a.product_type.cmp(&b.product_type))
            });
            let mut selected = std::collections::HashSet::new();
            for private in private_products {
                let Some(tier) = private
                    .product_type
                    .strip_prefix("membership:")
                    .filter(|tier| {
                        crate::payment::domain::product_kind::is_valid_membership_tier(tier)
                    })
                else {
                    continue;
                };
                if !selected.insert(tier.to_owned()) {
                    continue;
                }
                if let Some(option) = product
                    .products
                    .iter_mut()
                    .find(|p| p.target_tier.trim().eq_ignore_ascii_case(tier))
                {
                    option.private_price_cents = Some(private.price_cents);
                    option.title = private.name;
                    option.subtitle = "仅限指定账号".into();
                    option.description = format!("支付确认后开通 {tier}，权益按现有等级规则执行。");
                    option.membership_duration_months = Some(private.membership_duration_months);
                } else {
                    product
                        .products
                        .push(crate::payment::domain::order::MembershipProductOption {
                            target_tier: tier.to_owned(),
                            price_cents: private.price_cents,
                            title: private.name,
                            subtitle: "仅限指定账号".into(),
                            description: format!("支付确认后开通 {tier}，权益按现有等级规则执行。"),
                            membership_duration_months: Some(private.membership_duration_months),
                            private_price_cents: Some(private.price_cents),
                            presentation: Default::default(),
                        });
                }
            }
        }
        Ok(product)
    }

    pub async fn execute_catalog_for_user(
        &self,
        user_id: Option<uuid::Uuid>,
        current_tier: Option<&str>,
        capabilities: &[String],
    ) -> anyhow::Result<MembershipProduct> {
        self.catalog_for_context(user_id, current_tier, capabilities, None)
            .await
    }
    pub async fn execute_purchase_catalog(
        &self,
        user_id: uuid::Uuid,
        current_tier: Option<&str>,
        capabilities: &[String],
        target: &str,
    ) -> anyhow::Result<MembershipProduct> {
        self.catalog_for_context(Some(user_id), current_tier, capabilities, Some(target))
            .await
    }
    async fn catalog_for_context(
        &self,
        user_id: Option<uuid::Uuid>,
        current_tier: Option<&str>,
        capabilities: &[String],
        target: Option<&str>,
    ) -> anyhow::Result<MembershipProduct> {
        use crate::payment::domain::{
            order::{MembershipPresentation, MembershipProductOption},
            product_kind::{ProductSpecError, UserContext},
        };
        let mut product = self.execute_for_user(user_id).await?;
        let context = UserContext {
            current_tier: current_tier.unwrap_or("V1").into(),
            capabilities: capabilities.to_vec(),
        };
        let registry = &self.registry;
        let mut rows = self
            .products
            .membership_catalog_products(user_id.unwrap_or_default())
            .await?;
        rows.sort_by(|a, b| {
            a.sort_order
                .cmp(&b.sort_order)
                .then(a.price_cents.cmp(&b.price_cents))
                .then(a.product_key.cmp(&b.product_key))
        });
        let mut added = std::collections::HashSet::new();
        let mut additions = Vec::new();
        let mut target_error = None;
        for spec in rows.into_iter().filter(|spec| spec.is_table_offer()) {
            if let Err(error) = registry.validate(&spec) {
                tracing::warn!(%error,"invalid membership catalog offer omitted");
                continue;
            }
            let snapshot = crate::payment::domain::product_kind::GrantSnapshot::from_spec(&spec)?;
            let grant = snapshot.membership()?;
            if target.is_some_and(|target| target == grant.tier)
                && !registry.eligible(&spec, &context)
            {
                target_error = Some(if !spec.capability_allowed(capabilities) {
                    ProductSpecError::CapabilityRequired
                } else {
                    ProductSpecError::Ineligible
                });
            }
            if !registry.eligible(&spec, &context) || !added.insert(grant.tier.clone()) {
                continue;
            }
            let is_pass = spec.is_pass();
            let months = match grant.expiry {
                crate::payment::domain::product_kind::MembershipExpiry::Months { months } => {
                    Some(months)
                }
                _ => None,
            };
            product.products.retain(|p| p.target_tier != grant.tier);
            additions.push(MembershipProductOption {
                target_tier: grant.tier,
                price_cents: spec.price_cents,
                title: spec.title.clone().unwrap_or_default(),
                subtitle: spec.subtitle.clone().unwrap_or_default(),
                description: spec.description.clone().unwrap_or_default(),
                membership_duration_months: months,
                private_price_cents: None,
                presentation: MembershipPresentation {
                    expiry_rule: Some(if is_pass { "end_of_next_day" } else { "months" }.into()),
                    duration_label: if is_pass {
                        Some("购买次日 23:59（北京时间）".into())
                    } else {
                        None
                    },
                    group_key: spec.group_key.clone(),
                    is_pass: Some(is_pass),
                    spec: Some(spec),
                },
            });
        }
        if target.is_some_and(|tier| !added.contains(tier) && product.find_option(tier).is_none()) {
            if let Some(error) = target_error {
                return Err(error.into());
            }
        }
        additions.extend(product.products);
        product.products = additions;
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
            if option.presentation.spec.is_some() {
                continue;
            }
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
                .unwrap_or(option.private_price_cents.unwrap_or(option.price_cents));
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
