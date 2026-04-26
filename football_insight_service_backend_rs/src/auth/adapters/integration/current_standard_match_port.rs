use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::auth::ports::current_standard_match_port::CurrentStandardMatchPort;

const DEFAULT_TICKET_MONITOR_BASE_URL: &str = "http://127.0.0.1:4000";

pub struct HttpCurrentStandardMatchPort {
    client: Client,
    endpoint: String,
}

impl HttpCurrentStandardMatchPort {
    pub fn new(base_url: Option<String>) -> Self {
        let normalized_base_url = base_url
            .unwrap_or_else(|| DEFAULT_TICKET_MONITOR_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();

        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .expect("failed to build ticket monitor http client"),
            endpoint: format!("{normalized_base_url}/api/matches/current-standard"),
        }
    }
}

#[async_trait]
impl CurrentStandardMatchPort for HttpCurrentStandardMatchPort {
    async fn fetch_current_match_id(&self) -> anyhow::Result<Option<String>> {
        let response = self
            .client
            .get(&self.endpoint)
            .send()
            .await
            .with_context(|| format!("failed to call ticket monitor endpoint: {}", self.endpoint))?
            .error_for_status()
            .with_context(|| {
                format!(
                    "ticket monitor endpoint returned non-success: {}",
                    self.endpoint
                )
            })?;

        let payload: CurrentStandardMatchResponse = response
            .json()
            .await
            .context("failed to parse ticket monitor current-standard response")?;

        Ok(payload
            .match_info
            .and_then(|match_info| match_info.match_id))
    }
}

#[derive(Debug, Deserialize)]
struct CurrentStandardMatchResponse {
    #[serde(default)]
    match_info: Option<CurrentStandardMatchInfo>,
}

#[derive(Debug, Deserialize)]
struct CurrentStandardMatchInfo {
    #[serde(default)]
    match_id: Option<String>,
}
