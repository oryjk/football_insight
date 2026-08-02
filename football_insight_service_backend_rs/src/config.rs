use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabasePoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub idle_timeout_secs: u64,
    pub max_lifetime_secs: u64,
}

impl DatabasePoolConfig {
    pub fn from_env() -> Result<Self> {
        Self::from_values(
            std::env::var("DB_POOL_MAX_CONNECTIONS").ok(),
            std::env::var("DB_POOL_MIN_CONNECTIONS").ok(),
            std::env::var("DB_POOL_ACQUIRE_TIMEOUT_SECS").ok(),
            std::env::var("DB_POOL_IDLE_TIMEOUT_SECS").ok(),
            std::env::var("DB_POOL_MAX_LIFETIME_SECS").ok(),
        )
    }

    fn from_values(
        max_connections: Option<String>,
        min_connections: Option<String>,
        acquire_timeout_secs: Option<String>,
        idle_timeout_secs: Option<String>,
        max_lifetime_secs: Option<String>,
    ) -> Result<Self> {
        let max_connections = parse_u32_config(max_connections, "DB_POOL_MAX_CONNECTIONS", 10)?;
        let min_connections = parse_u32_config(min_connections, "DB_POOL_MIN_CONNECTIONS", 3)?;
        let acquire_timeout_secs =
            parse_u64_config(acquire_timeout_secs, "DB_POOL_ACQUIRE_TIMEOUT_SECS", 5)?;
        let idle_timeout_secs =
            parse_u64_config(idle_timeout_secs, "DB_POOL_IDLE_TIMEOUT_SECS", 90)?;
        let max_lifetime_secs =
            parse_u64_config(max_lifetime_secs, "DB_POOL_MAX_LIFETIME_SECS", 600)?;

        if min_connections > max_connections {
            anyhow::bail!("DB_POOL_MIN_CONNECTIONS must be <= DB_POOL_MAX_CONNECTIONS");
        }

        Ok(Self {
            max_connections,
            min_connections,
            acquire_timeout_secs,
            idle_timeout_secs,
            max_lifetime_secs,
        })
    }
}

fn parse_u32_config(value: Option<String>, key: &str, default: u32) -> Result<u32> {
    match value.map(|item| item.trim().to_string()) {
        None => Ok(default),
        Some(value) if value.is_empty() => Ok(default),
        Some(value) => value
            .parse::<u32>()
            .with_context(|| format!("{key} must be a valid u32")),
    }
}

fn parse_u64_config(value: Option<String>, key: &str, default: u64) -> Result<u64> {
    match value.map(|item| item.trim().to_string()) {
        None => Ok(default),
        Some(value) if value.is_empty() => Ok(default),
        Some(value) => value
            .parse::<u64>()
            .with_context(|| format!("{key} must be a valid u64")),
    }
}

pub struct AppConfig {
    pub port: u16,
    pub database_url: String,
    pub database_pool: DatabasePoolConfig,
    pub jwt_secret: String,
    pub openai_api_key: Option<String>,
    pub openai_base_url: Option<String>,
    pub ai_chat_model: String,
    pub wechat_app_id: String,
    pub wechat_app_secret: String,
    pub wechat_mini_app_id: String,
    pub wechat_mini_app_secret: String,
    pub wechat_webhook_token: String,
    pub wechat_encoding_aes_key: String,
    pub ticket_monitor_base_url: Option<String>,
    pub redis_url: String,
    pub wechat_pay_mch_id: String,
    pub wechat_pay_api_key: String,
    pub public_base_url: String,
    pub reflux_notification_worker: RefluxNotificationWorkerConfig,
    pub smtp_email: Option<SmtpEmailConfig>,
    pub minio: Option<MinioConfig>,
    pub jpush: Option<JPushConfig>,
    pub seat_swap_mini_subscribe_template_id: String,
    pub seat_swap_mini_subscribe_page: String,
    pub admin_jwt_secret: String,
    pub admin_owner_username: String,
    pub admin_owner_password: String,
    pub admin_owner_display_name: String,
}

#[derive(Debug, Clone)]
pub struct JPushConfig {
    pub app_key: String,
    pub master_secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefluxNotificationWorkerConfig {
    pub enabled: bool,
    pub poll_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtpEmailConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinioConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub region: String,
    pub prefix: String,
    pub public_base_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("PORT must be a valid u16")?;

        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL is required")?;
        let database_pool = DatabasePoolConfig::from_env()?;

        let jwt_secret = std::env::var("JWT_SECRET").context("JWT_SECRET is required")?;
        let openai_api_key = std::env::var("OPENAI_API_KEY").ok();
        let openai_base_url = std::env::var("OPENAI_BASE_URL")
            .ok()
            .or_else(|| Some("https://open.bigmodel.cn/api/coding/paas/v4".to_string()));
        let ai_chat_model =
            std::env::var("AI_CHAT_MODEL").unwrap_or_else(|_| "glm-5.1".to_string());
        let wechat_app_id = std::env::var("WECHAT_APP_ID").context("WECHAT_APP_ID is required")?;
        let wechat_app_secret =
            std::env::var("WECHAT_APP_SECRET").context("WECHAT_APP_SECRET is required")?;
        let wechat_mini_app_id =
            std::env::var("WECHAT_MINI_APP_ID").unwrap_or_else(|_| wechat_app_id.clone());
        let wechat_mini_app_secret =
            std::env::var("WECHAT_MINI_APP_SECRET").unwrap_or_else(|_| wechat_app_secret.clone());
        let wechat_webhook_token =
            std::env::var("WECHAT_WEBHOOK_TOKEN").context("WECHAT_WEBHOOK_TOKEN is required")?;
        let wechat_encoding_aes_key = std::env::var("WECHAT_ENCODING_AES_KEY")
            .context("WECHAT_ENCODING_AES_KEY is required")?;
        let ticket_monitor_base_url = std::env::var("TICKET_MONITOR_BASE_URL").ok();
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let wechat_pay_mch_id = std::env::var("WECHAT_PAY_MCH_ID").unwrap_or_default();
        let wechat_pay_api_key = std::env::var("WECHAT_PAY_API_KEY").unwrap_or_default();
        let public_base_url = std::env::var("PUBLIC_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
        let reflux_notification_worker = RefluxNotificationWorkerConfig {
            enabled: parse_bool_env("FI_REFLUX_NOTIFICATION_WORKER_ENABLED", false),
            poll_seconds: std::env::var("FI_REFLUX_NOTIFICATION_POLL_SECONDS")
                .ok()
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(60),
        };
        let smtp_email = SmtpEmailConfig::from_env();
        let minio = MinioConfig::from_env();
        let jpush = JPushConfig::from_env();
        let seat_swap_mini_subscribe_template_id =
            std::env::var("WECHAT_MINI_SEAT_SWAP_SUBSCRIBE_TEMPLATE_ID").unwrap_or_default();
        let seat_swap_mini_subscribe_page = std::env::var("WECHAT_MINI_SEAT_SWAP_SUBSCRIBE_PAGE")
            .unwrap_or_else(|_| "pages/seat-swap/index".to_string());
        let admin_jwt_secret =
            std::env::var("ADMIN_JWT_SECRET").context("ADMIN_JWT_SECRET is required")?;
        if admin_jwt_secret.trim().len() < 32 {
            anyhow::bail!("ADMIN_JWT_SECRET must be at least 32 characters");
        }
        let admin_owner_username =
            std::env::var("ADMIN_OWNER_USERNAME").context("ADMIN_OWNER_USERNAME is required")?;
        let admin_owner_password =
            std::env::var("ADMIN_OWNER_PASSWORD").context("ADMIN_OWNER_PASSWORD is required")?;
        let admin_owner_display_name = std::env::var("ADMIN_OWNER_DISPLAY_NAME")
            .unwrap_or_else(|_| "Football Insight Owner".to_string());

        Ok(Self {
            port,
            database_url,
            database_pool,
            jwt_secret,
            openai_api_key,
            openai_base_url,
            ai_chat_model,
            wechat_app_id,
            wechat_app_secret,
            wechat_mini_app_id,
            wechat_mini_app_secret,
            wechat_webhook_token,
            wechat_encoding_aes_key,
            ticket_monitor_base_url,
            redis_url,
            wechat_pay_mch_id,
            wechat_pay_api_key,
            public_base_url,
            reflux_notification_worker,
            smtp_email,
            minio,
            jpush,
            seat_swap_mini_subscribe_template_id,
            seat_swap_mini_subscribe_page,
            admin_jwt_secret,
            admin_owner_username,
            admin_owner_password,
            admin_owner_display_name,
        })
    }
}

impl MinioConfig {
    fn from_env() -> Option<Self> {
        let endpoint = std::env::var("FI_MINIO_ENDPOINT").ok()?.trim().to_string();
        let access_key = std::env::var("FI_MINIO_ACCESS_KEY")
            .ok()?
            .trim()
            .to_string();
        let secret_key = std::env::var("FI_MINIO_SECRET_KEY").ok()?;
        let bucket = std::env::var("FI_MINIO_BUCKET")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "football-insight".to_string());
        let region = std::env::var("FI_MINIO_REGION")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "us-east-1".to_string());
        let prefix = std::env::var("FI_MINIO_PREFIX")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "seat-swap".to_string());
        let public_base_url = std::env::var("FI_MINIO_PUBLIC_BASE_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| {
                format!(
                    "{}/{}",
                    endpoint.trim_end_matches('/'),
                    bucket.trim_matches('/')
                )
            });

        if endpoint.is_empty() || access_key.is_empty() || secret_key.is_empty() {
            return None;
        }

        Some(Self {
            endpoint,
            access_key,
            secret_key,
            bucket,
            region,
            prefix,
            public_base_url,
        })
    }
}

impl JPushConfig {
    fn from_env() -> Option<Self> {
        let app_key = std::env::var("JPUSH_APP_KEY").ok()?.trim().to_string();
        let master_secret = std::env::var("JPUSH_MASTER_SECRET").ok()?;
        if app_key.is_empty() || master_secret.trim().is_empty() {
            return None;
        }
        Some(Self {
            app_key,
            master_secret,
        })
    }
}

impl SmtpEmailConfig {
    fn from_env() -> Option<Self> {
        let host = std::env::var("FI_SMTP_HOST").ok()?.trim().to_string();
        let username = std::env::var("FI_SMTP_USERNAME").ok()?.trim().to_string();
        let password = std::env::var("FI_SMTP_PASSWORD").ok()?;
        let from = std::env::var("FI_SMTP_FROM")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| username.clone());
        let port = std::env::var("FI_SMTP_PORT")
            .ok()
            .and_then(|value| value.trim().parse::<u16>().ok())
            .unwrap_or(465);

        if host.is_empty() || username.is_empty() || password.is_empty() {
            return None;
        }

        Some(Self {
            host,
            port,
            username,
            password,
            from,
        })
    }
}

fn parse_bool_env(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::DatabasePoolConfig;

    #[test]
    fn database_pool_config_uses_defaults_for_missing_or_blank_values() {
        let config = DatabasePoolConfig::from_values(
            None,
            Some("".to_string()),
            None,
            Some("   ".to_string()),
            None,
        )
        .expect("pool config should resolve");

        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 3);
        assert_eq!(config.acquire_timeout_secs, 5);
        assert_eq!(config.idle_timeout_secs, 90);
        assert_eq!(config.max_lifetime_secs, 600);
    }

    #[test]
    fn database_pool_config_reads_explicit_values() {
        let config = DatabasePoolConfig::from_values(
            Some("12".to_string()),
            Some("4".to_string()),
            Some("7".to_string()),
            Some("45".to_string()),
            Some("300".to_string()),
        )
        .expect("pool config should resolve");

        assert_eq!(config.max_connections, 12);
        assert_eq!(config.min_connections, 4);
        assert_eq!(config.acquire_timeout_secs, 7);
        assert_eq!(config.idle_timeout_secs, 45);
        assert_eq!(config.max_lifetime_secs, 300);
    }

    #[test]
    fn database_pool_config_rejects_invalid_or_inverted_values() {
        assert!(
            DatabasePoolConfig::from_values(Some("bad".to_string()), None, None, None, None,)
                .is_err()
        );

        assert!(
            DatabasePoolConfig::from_values(
                Some("2".to_string()),
                Some("3".to_string()),
                None,
                None,
                None,
            )
            .is_err()
        );
    }
}
