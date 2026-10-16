use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RecordPageActivityRequest {
    pub page_key: String,
}

#[derive(Debug, Deserialize)]
pub struct PaywallEventsRequest {
    #[serde(default)]
    pub events: Vec<PaywallEventRequest>,
}

#[derive(Debug, Deserialize)]
pub struct PaywallEventRequest {
    pub surface: String,
    pub action: String,
    pub membership_tier: Option<String>,
    #[serde(default)]
    pub detail: serde_json::Value,
}
