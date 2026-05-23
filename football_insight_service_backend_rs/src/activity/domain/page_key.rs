#[derive(Debug, thiserror::Error)]
pub enum ActivityError {
    #[error("页面标识不支持")]
    UnsupportedPageKey,
}

pub const ALLOWED_ACTIVITY_PAGE_KEYS: &[&str] = &[
    "home",
    "rankings",
    "insights",
    "ticket_watch",
    "user",
    "membership_purchase",
];

pub fn validate_activity_page_key(page_key: &str) -> Result<String, ActivityError> {
    let normalized = page_key.trim();
    if ALLOWED_ACTIVITY_PAGE_KEYS.contains(&normalized) {
        return Ok(normalized.to_string());
    }

    Err(ActivityError::UnsupportedPageKey)
}
