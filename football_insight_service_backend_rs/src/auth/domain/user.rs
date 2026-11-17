use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub account_identifier: String,
    pub display_name: Option<String>,
    pub invite_code: Option<String>,
    pub avatar_url: Option<String>,
    pub has_wechat_binding: bool,
    pub membership_tier: String,
    pub membership_expires_at: Option<DateTime<Utc>>,
    pub membership_benefits_enabled: bool,
    pub ticket_watch_poll_interval_seconds: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct StoredAuthUser {
    pub user: AuthUser,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct AuthTokenBundle {
    pub user: AuthUser,
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}

pub fn normalize_account_identifier(input: &str) -> String {
    input.trim().to_string()
}

pub fn normalize_password(input: &str) -> String {
    input.trim().to_string()
}

pub fn validate_account_identifier(input: &str) -> anyhow::Result<String> {
    let normalized = normalize_account_identifier(input);
    let is_phone_number = normalized.len() == 11
        && normalized.starts_with('1')
        && normalized.chars().all(|char| char.is_ascii_digit());
    let char_count = normalized.chars().count();

    if is_phone_number || char_count > 5 {
        return Ok(normalized);
    }

    anyhow::bail!(
        "account identifier must be a valid 11-digit mainland China mobile number or a username longer than 5 characters"
    );
}

pub fn is_mainland_mobile(input: &str) -> bool {
    let normalized = input.trim();
    normalized.len() == 11
        && normalized.starts_with('1')
        && normalized.bytes().all(|byte| byte.is_ascii_digit())
}

pub fn validate_phone_number(input: &str) -> anyhow::Result<String> {
    let normalized = normalize_account_identifier(input);
    if !is_mainland_mobile(&normalized) {
        anyhow::bail!("phone number must be a valid 11-digit mainland China mobile number");
    }

    Ok(normalized)
}

pub fn validate_custom_nickname(input: &str) -> anyhow::Result<String> {
    let normalized = input.trim();
    let char_count = normalized.chars().count();
    if !(2..=16).contains(&char_count) {
        anyhow::bail!("nickname must be 2 to 16 characters");
    }
    if normalized
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '<' | '>'))
    {
        anyhow::bail!("nickname contains unsupported characters");
    }
    if is_mainland_mobile(normalized) {
        anyhow::bail!("nickname cannot be a phone number");
    }
    if normalized == "微信用户" || normalized.to_ascii_lowercase().starts_with("wx_") {
        anyhow::bail!("please set a custom nickname");
    }
    Ok(normalized.to_string())
}

pub fn has_valid_avatar_url(avatar_url: Option<&str>) -> bool {
    let Some(raw) = avatar_url.map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        return raw.len() > 8;
    }
    const DATA_PREFIXES: [&str; 4] = [
        "data:image/jpeg;base64,",
        "data:image/jpg;base64,",
        "data:image/png;base64,",
        "data:image/webp;base64,",
    ];
    DATA_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix) && raw.len() > prefix.len())
}

pub fn resolve_profile_completed(
    display_name: Option<&str>,
    account_identifier: &str,
    avatar_url: Option<&str>,
    custom_profile_recorded: Option<bool>,
) -> bool {
    if !has_valid_avatar_url(avatar_url) {
        return false;
    }
    let Ok(nickname) = validate_custom_nickname(display_name.unwrap_or_default()) else {
        return false;
    };
    custom_profile_recorded == Some(true) || nickname != account_identifier.trim()
}

pub fn sanitize_public_display_name(display_name: Option<&str>) -> String {
    let Some(trimmed) = display_name.map(str::trim).filter(|value| !value.is_empty()) else {
        return "球迷".to_string();
    };
    if trimmed.to_ascii_lowercase().starts_with("wx_") {
        return "球迷".to_string();
    }
    if is_mainland_mobile(trimmed) {
        return format!("{}****{}", &trimmed[..3], &trimmed[7..]);
    }
    trimmed.to_string()
}

pub fn resolve_membership_benefits_enabled(
    official_wechat_open_id: Option<&str>,
    has_active_follow: Option<bool>,
) -> bool {
    let has_official_binding = official_wechat_open_id
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());

    if !has_official_binding {
        return true;
    }

    has_active_follow.unwrap_or(false)
}

pub fn requires_official_wechat_binding(
    bound: bool,
    membership_origin: &str,
    expires_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    let active_purchase = is_active_purchase(membership_origin, expires_at, now);
    !bound && !active_purchase
}

fn is_active_purchase(origin: &str, expiry: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    origin == "purchase" && expiry.is_none_or(|at| at > now)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialWechatAccess {
    pub bound: bool,
    pub subscribed: Option<bool>,
    pub binding_required: bool,
    pub benefits_enabled: bool,
}

pub fn resolve_official_wechat_access(
    bound: bool,
    subscribed: Option<bool>,
    origin: &str,
    expiry: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> OfficialWechatAccess {
    let paid = is_active_purchase(origin, expiry, now);
    OfficialWechatAccess {
        bound,
        subscribed: if bound { subscribed } else { None },
        binding_required: !bound && !paid,
        benefits_enabled: paid || (bound && subscribed == Some(true)),
    }
}

pub fn resolve_effective_membership_tier(
    membership_tier: &str,
    membership_expires_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> String {
    if membership_expires_at.is_some_and(|expires_at| expires_at <= now) {
        return "V3".to_string();
    }

    membership_tier.to_string()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{resolve_effective_membership_tier, resolve_membership_benefits_enabled};

    #[test]
    fn only_active_purchase_memberships_are_exempt_from_official_binding() {
        let now = Utc.with_ymd_and_hms(2026, 10, 20, 12, 0, 0).unwrap();
        for (bound, origin, expiry, required) in [
            (
                false,
                "referral",
                Some(now + chrono::Duration::days(1)),
                true,
            ),
            (false, "unknown", None, true),
            (
                false,
                "purchase",
                Some(now + chrono::Duration::seconds(1)),
                false,
            ),
            (false, "purchase", None, false),
            (false, "purchase", Some(now), true),
            (
                false,
                "purchase",
                Some(now - chrono::Duration::seconds(1)),
                true,
            ),
            (true, "unknown", None, false),
        ] {
            assert_eq!(
                super::requires_official_wechat_binding(bound, origin, expiry, now),
                required
            );
        }
    }

    #[test]
    fn membership_benefits_stay_enabled_without_official_binding() {
        assert!(resolve_membership_benefits_enabled(None, None));
        assert!(resolve_membership_benefits_enabled(Some(""), Some(false)));
    }

    #[test]
    fn membership_benefits_require_active_follow_when_binding_exists() {
        assert!(resolve_membership_benefits_enabled(
            Some("official-open-id"),
            Some(true)
        ));
        assert!(!resolve_membership_benefits_enabled(
            Some("official-open-id"),
            Some(false),
        ));
        assert!(!resolve_membership_benefits_enabled(
            Some("official-open-id"),
            None,
        ));
    }

    #[test]
    fn expiring_membership_tier_falls_back_to_v3_after_expiration() {
        let now = Utc.with_ymd_and_hms(2026, 4, 24, 12, 0, 0).unwrap();

        assert_eq!(
            resolve_effective_membership_tier(
                "V8",
                Some(Utc.with_ymd_and_hms(2026, 4, 24, 11, 59, 59).unwrap()),
                now,
            ),
            "V3"
        );
        assert_eq!(
            resolve_effective_membership_tier(
                "V8",
                Some(Utc.with_ymd_and_hms(2026, 4, 24, 12, 0, 1).unwrap()),
                now,
            ),
            "V8"
        );
        assert_eq!(resolve_effective_membership_tier("V8", None, now), "V8");
        assert_eq!(
            resolve_effective_membership_tier(
                "V4",
                Some(Utc.with_ymd_and_hms(2026, 4, 24, 11, 59, 59).unwrap()),
                now,
            ),
            "V3"
        );
    }

    #[test]
    fn custom_nickname_rejects_phones_defaults_control_chars_and_length() {
        assert_eq!(
            super::validate_custom_nickname("  蓉城老球迷  ").unwrap(),
            "蓉城老球迷"
        );
        assert!(super::validate_custom_nickname("a").is_err());
        assert!(super::validate_custom_nickname(&"迷".repeat(17)).is_err());
        assert!(super::validate_custom_nickname("13800138000").is_err());
        assert!(super::validate_custom_nickname("微信用户").is_err());
        assert!(super::validate_custom_nickname("wx_abcdef").is_err());
        assert!(super::validate_custom_nickname("球迷\n换行").is_err());
        assert!(super::validate_custom_nickname("球迷<script>").is_err());
    }

    #[test]
    fn resolve_profile_completed_blocks_unmodified_defaults_and_phone_numbers() {
        let avatar = Some("https://oryjk.cn:82/minio/football-insight/u.jpg");
        assert!(!super::resolve_profile_completed(
            Some("13800138000"),
            "13800138000",
            avatar,
            Some(false)
        ));
        assert!(!super::resolve_profile_completed(
            Some("footballfan"),
            "footballfan",
            avatar,
            Some(false)
        ));
        assert!(super::resolve_profile_completed(
            Some("footballfan"),
            "footballfan",
            avatar,
            Some(true)
        ));
        assert!(super::resolve_profile_completed(
            Some("蓉城12号"),
            "13800138000",
            avatar,
            None
        ));
        assert!(!super::resolve_profile_completed(
            Some("蓉城12号"),
            "13800138000",
            None,
            Some(true)
        ));
    }

    #[test]
    fn sanitize_public_display_name_masks_mobile_and_wx_identifiers() {
        assert_eq!(
            super::sanitize_public_display_name(Some("13800138000")),
            "138****8000"
        );
        assert_eq!(
            super::sanitize_public_display_name(Some("wx_o123456")),
            "球迷"
        );
        assert_eq!(super::sanitize_public_display_name(None), "球迷");
        assert_eq!(
            super::sanitize_public_display_name(Some(" 凤凰山南看台 ")),
            "凤凰山南看台"
        );
    }
}
