use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::activity::domain::paywall::valid_surface;

const MAX_RANGE_DAYS: i64 = 92;
const DEFAULT_RANGE_DAYS: i64 = 7;

#[derive(Debug, Clone)]
pub struct PaywallRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub surface: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AdminPaywallError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("paywall measurement query failed")]
    Internal,
}

impl PaywallRange {
    /// `from`/`to` are optional RFC3339 timestamps; the default is the last 7 days ending at `now`.
    pub fn parse(
        from: Option<&str>,
        to: Option<&str>,
        surface: Option<String>,
        now: DateTime<Utc>,
    ) -> Result<Self, AdminPaywallError> {
        let parse = |value: &str| {
            DateTime::parse_from_rfc3339(value)
                .map(|v| v.to_utc())
                .map_err(|_| AdminPaywallError::InvalidInput("from and to must be RFC3339 timestamps".into()))
        };
        let to = to.map(parse).transpose()?.unwrap_or(now);
        let from = from
            .map(parse)
            .transpose()?
            .unwrap_or(to - Duration::days(DEFAULT_RANGE_DAYS));
        if from >= to || to - from > Duration::days(MAX_RANGE_DAYS) {
            return Err(AdminPaywallError::InvalidInput(
                "time range must be positive and at most 92 days".into(),
            ));
        }
        if surface.as_deref().is_some_and(|value| !valid_surface(value)) {
            return Err(AdminPaywallError::InvalidInput("invalid paywall surface".into()));
        }
        Ok(Self { from, to, surface })
    }
}

/// Per Beijing calendar day and surface.
#[derive(Debug, Clone)]
pub struct PaywallFunnelRow {
    pub day: NaiveDate,
    pub surface: String,
    pub impressions: i64,
    pub impression_users: i64,
    pub taps: i64,
    pub tap_users: i64,
}

#[derive(Debug, Clone)]
pub struct PaywallTierRow {
    pub membership_tier: Option<String>,
    pub surface: String,
    pub impression_users: i64,
    pub tap_users: i64,
}

/// Share of users who saw a paywall and tapped it; undefined (None) without any impression.
pub fn tap_rate(tap_users: i64, impression_users: i64) -> Option<f64> {
    (impression_users > 0).then(|| tap_users as f64 / impression_users as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 20, 12, 0, 0).unwrap()
    }

    #[test]
    fn defaults_to_the_last_seven_days() {
        let range = PaywallRange::parse(None, None, None, now()).unwrap();
        assert_eq!(range.to, now());
        assert_eq!(range.from, now() - Duration::days(7));
        assert!(range.surface.is_none());
    }

    #[test]
    fn accepts_explicit_bounds_and_a_known_surface() {
        let range = PaywallRange::parse(
            Some("2026-10-01T00:00:00+08:00"),
            Some("2026-10-10T00:00:00Z"),
            Some("user_membership_entry".into()),
            now(),
        )
        .unwrap();
        assert_eq!(range.from, Utc.with_ymd_and_hms(2026, 9, 30, 16, 0, 0).unwrap());
        assert_eq!(range.surface.as_deref(), Some("user_membership_entry"));
    }

    #[test]
    fn rejects_inverted_oversized_malformed_ranges_and_unknown_surfaces() {
        assert!(PaywallRange::parse(Some("2026-10-10T00:00:00Z"), Some("2026-10-01T00:00:00Z"), None, now()).is_err());
        assert!(PaywallRange::parse(Some("2026-01-01T00:00:00Z"), Some("2026-10-01T00:00:00Z"), None, now()).is_err());
        assert!(PaywallRange::parse(Some("yesterday"), None, None, now()).is_err());
        assert!(PaywallRange::parse(None, None, Some("home".into()), now()).is_err());
    }

    #[test]
    fn exactly_ninety_two_days_is_allowed() {
        let to = "2026-10-20T00:00:00Z";
        let from = "2026-07-20T00:00:00Z";
        assert!(PaywallRange::parse(Some(from), Some(to), None, now()).is_ok());
    }

    #[test]
    fn tap_rate_is_undefined_without_impressions() {
        assert_eq!(tap_rate(3, 0), None);
        assert_eq!(tap_rate(0, 4), Some(0.0));
        assert_eq!(tap_rate(1, 4), Some(0.25));
    }
}
