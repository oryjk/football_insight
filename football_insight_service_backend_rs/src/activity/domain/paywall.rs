use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use uuid::Uuid;

/// The only places a paywall can be measured. Anything else is dropped, never stored.
pub const PAYWALL_SURFACES: &[&str] = &[
    "ticket_watch_recent_reflux_lock",
    "ticket_watch_match_id_sheet",
    "insights_ticket_watch_entry",
    "user_membership_entry",
];

const MAX_DETAIL_BYTES: usize = 512;

pub fn valid_surface(surface: &str) -> bool {
    PAYWALL_SURFACES.contains(&surface)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaywallAction {
    Impression,
    Tap,
}

impl PaywallAction {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "impression" => Some(Self::Impression),
            "tap" => Some(Self::Tap),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Impression => "impression",
            Self::Tap => "tap",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewPaywallEvent {
    pub user_id: Uuid,
    pub surface: String,
    pub action: PaywallAction,
    pub membership_tier: Option<String>,
    pub detail: Value,
    pub occurred_at: DateTime<Utc>,
}

/// The tier is a measurement label reported by the client; only the `V1`..`V9` shape is kept.
pub fn normalize_tier(value: Option<&str>) -> Option<String> {
    let tier = value?.trim().to_uppercase();
    let digits = tier.strip_prefix('V')?;
    (digits.len() == 1 && matches!(digits.as_bytes()[0], b'1'..=b'9')).then_some(tier)
}

/// Keeps a fixed allowlist of small scalar keys; anything else (or an oversized result) is dropped.
pub fn sanitize_detail(value: &Value) -> Value {
    let Some(input) = value.as_object() else {
        return Value::Object(Map::new());
    };
    let mut output = Map::new();
    for key in ["mini_version", "platform"] {
        if let Some(text) = input.get(key).and_then(Value::as_str) {
            let clipped: String = text.chars().take(40).collect();
            output.insert(key.into(), Value::String(clipped));
        }
    }
    if let Some(match_id) = input.get("match_id") {
        let parsed = match_id
            .as_i64()
            .or_else(|| match_id.as_str().and_then(|s| s.trim().parse::<i64>().ok()));
        if let Some(id) = parsed.filter(|id| *id > 0) {
            output.insert("match_id".into(), Value::from(id));
        }
    }
    let sanitized = Value::Object(output);
    if sanitized.to_string().len() > MAX_DETAIL_BYTES {
        return Value::Object(Map::new());
    }
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_the_four_known_surfaces_are_valid() {
        for surface in PAYWALL_SURFACES {
            assert!(valid_surface(surface));
        }
        assert!(!valid_surface("home"));
        assert!(!valid_surface(""));
        assert!(!valid_surface("ticket_watch_recent_reflux_lock "));
    }

    #[test]
    fn actions_are_a_closed_set() {
        assert_eq!(PaywallAction::parse("impression"), Some(PaywallAction::Impression));
        assert_eq!(PaywallAction::parse("tap"), Some(PaywallAction::Tap));
        assert_eq!(PaywallAction::parse("click"), None);
        assert_eq!(PaywallAction::Tap.as_str(), "tap");
    }

    #[test]
    fn tier_keeps_only_the_v1_to_v9_shape() {
        assert_eq!(normalize_tier(Some(" v3 ")).as_deref(), Some("V3"));
        assert_eq!(normalize_tier(Some("V9")).as_deref(), Some("V9"));
        for bad in ["V0", "V10", "VIP", "3", "", "V3; drop table"] {
            assert_eq!(normalize_tier(Some(bad)), None, "{bad}");
        }
        assert_eq!(normalize_tier(None), None);
    }

    #[test]
    fn detail_keeps_only_allowlisted_scalars() {
        let cleaned = sanitize_detail(&json!({
            "mini_version": "1.0.77", "platform": "android", "match_id": "583",
            "token": "secret", "openid": "o123", "nested": {"a": 1}
        }));
        assert_eq!(cleaned, json!({"mini_version": "1.0.77", "platform": "android", "match_id": 583}));
    }

    #[test]
    fn detail_rejects_non_objects_bad_ids_and_clips_long_text() {
        assert_eq!(sanitize_detail(&json!("text")), json!({}));
        assert_eq!(sanitize_detail(&json!({"match_id": -5})), json!({}));
        assert_eq!(sanitize_detail(&json!({"match_id": "abc"})), json!({}));
        let long = "x".repeat(500);
        let cleaned = sanitize_detail(&json!({"platform": long}));
        assert_eq!(cleaned["platform"].as_str().unwrap().len(), 40);
    }
}
