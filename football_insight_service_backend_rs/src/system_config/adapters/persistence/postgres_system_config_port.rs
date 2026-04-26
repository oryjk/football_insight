use async_trait::async_trait;
use serde::Deserialize;
use sqlx::{PgPool, Row};

use crate::system_config::{
    domain::{
        ai_chat_config::AiChatSystemConfig,
        public_system_config::{
            AiChatMode, HomeBriefingMarquees, PublicSystemConfig, parse_membership_tier_rules,
        },
    },
    ports::system_config_port::SystemConfigPort,
};

pub struct PostgresSystemConfigPort {
    pool: PgPool,
}

impl PostgresSystemConfigPort {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, Deserialize, Default)]
struct HomeBriefingMarqueesPayload {
    #[serde(default)]
    leader: Vec<String>,
    #[serde(default)]
    scorer: Vec<String>,
    #[serde(default)]
    assist: Vec<String>,
}

pub(crate) fn parse_switch(value: Option<&str>) -> bool {
    value.is_some_and(|item| {
        matches!(
            item.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

pub(crate) fn parse_optional_config_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn parse_ai_chat_mode(value: Option<&str>) -> AiChatMode {
    AiChatMode::from_config_value(value)
}

fn clean_messages(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .take(5)
        .collect()
}

pub(crate) fn parse_home_briefing_marquees(value: Option<&str>) -> HomeBriefingMarquees {
    let Some(value) = value else {
        return HomeBriefingMarquees::default();
    };

    let Ok(payload) = serde_json::from_str::<HomeBriefingMarqueesPayload>(value) else {
        return HomeBriefingMarquees::default();
    };

    HomeBriefingMarquees {
        leader: clean_messages(payload.leader),
        scorer: clean_messages(payload.scorer),
        assist: clean_messages(payload.assist),
    }
}

#[async_trait]
impl SystemConfigPort for PostgresSystemConfigPort {
    async fn get_public_config(&self) -> anyhow::Result<PublicSystemConfig> {
        let rows = sqlx::query(
            r#"
            SELECT config_key, config_value
            FROM f_i_system_configs
            WHERE config_key IN (
                'wechat_login_enabled',
                'ai_chat_mode',
                'home_briefing_marquees',
                'membership_tier_rules'
            )
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let wechat_login_enabled = rows
            .iter()
            .find(|item| {
                item.try_get::<String, _>("config_key").ok().as_deref()
                    == Some("wechat_login_enabled")
            })
            .and_then(|item| item.try_get::<String, _>("config_value").ok())
            .as_deref()
            .map(|item| parse_switch(Some(item)))
            .unwrap_or(false);

        let ai_chat_mode = rows
            .iter()
            .find(|item| {
                item.try_get::<String, _>("config_key").ok().as_deref() == Some("ai_chat_mode")
            })
            .and_then(|item| item.try_get::<String, _>("config_value").ok());

        let home_briefing_marquees = rows
            .iter()
            .find(|item| {
                item.try_get::<String, _>("config_key").ok().as_deref()
                    == Some("home_briefing_marquees")
            })
            .and_then(|item| item.try_get::<String, _>("config_value").ok());

        let membership_tier_rules = rows
            .iter()
            .find(|item| {
                item.try_get::<String, _>("config_key").ok().as_deref()
                    == Some("membership_tier_rules")
            })
            .and_then(|item| item.try_get::<String, _>("config_value").ok());

        Ok(PublicSystemConfig::new(
            wechat_login_enabled,
            parse_ai_chat_mode(ai_chat_mode.as_deref()),
            parse_home_briefing_marquees(home_briefing_marquees.as_deref()),
            parse_membership_tier_rules(membership_tier_rules.as_deref()),
        ))
    }

    async fn get_ai_chat_config(&self) -> anyhow::Result<AiChatSystemConfig> {
        let rows = sqlx::query(
            r#"
            SELECT config_key, config_value
            FROM f_i_system_configs
            WHERE config_key IN (
                'ai_chat_model',
                'ai_chat_base_url'
            )
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let model = rows
            .iter()
            .find(|item| {
                item.try_get::<String, _>("config_key").ok().as_deref() == Some("ai_chat_model")
            })
            .and_then(|item| item.try_get::<String, _>("config_value").ok());

        let base_url = rows
            .iter()
            .find(|item| {
                item.try_get::<String, _>("config_key").ok().as_deref() == Some("ai_chat_base_url")
            })
            .and_then(|item| item.try_get::<String, _>("config_value").ok());

        Ok(AiChatSystemConfig::new(
            parse_optional_config_value(model.as_deref()),
            parse_optional_config_value(base_url.as_deref()),
        ))
    }

    async fn get_config_value(&self, config_key: &str) -> anyhow::Result<Option<String>> {
        let row = sqlx::query(
            r#"
            SELECT config_value
            FROM f_i_system_configs
            WHERE config_key = $1
            "#,
        )
        .bind(config_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row
            .and_then(|item| item.try_get::<String, _>("config_value").ok())
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty()))
    }
}
