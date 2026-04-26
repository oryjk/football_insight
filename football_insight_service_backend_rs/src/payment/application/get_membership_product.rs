use std::sync::Arc;

use crate::payment::domain::order::MembershipProduct;

pub struct GetMembershipProductUseCase {
    system_config_port: Arc<dyn crate::system_config::ports::system_config_port::SystemConfigPort>,
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;

    use super::GetMembershipProductUseCase;
    use crate::system_config::{
        domain::{ai_chat_config::AiChatSystemConfig, public_system_config::PublicSystemConfig},
        ports::system_config_port::SystemConfigPort,
    };

    struct FakeSystemConfigPort {
        value: Mutex<Option<String>>,
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
    async fn execute_returns_v6_to_v9_membership_products_by_default() {
        let use_case = GetMembershipProductUseCase::new(std::sync::Arc::new(
            FakeSystemConfigPort {
                value: Mutex::new(None),
            },
        ));

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
    ) -> Self {
        Self { system_config_port }
    }

    pub async fn execute(&self) -> anyhow::Result<MembershipProduct> {
        let config_value = self
            .system_config_port
            .get_config_value("membership_product")
            .await?;

        let product: MembershipProduct = match config_value {
            Some(value) => serde_json::from_str(&value).unwrap_or_else(|_| Self::default_product()),
            None => Self::default_product(),
        };

        Ok(product.normalized())
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
