pub const ALLOWED_ACTIVITY_PAGE_KEYS: &[&str] = &[
    "home",
    "rankings",
    "insights",
    "ticket_watch",
    "user",
    "membership_purchase",
];

pub fn validate_activity_page_key(page_key: &str) -> anyhow::Result<String> {
    let normalized = page_key.trim();
    if ALLOWED_ACTIVITY_PAGE_KEYS.contains(&normalized) {
        return Ok(normalized.to_string());
    }

    anyhow::bail!("unsupported activity page key")
}
