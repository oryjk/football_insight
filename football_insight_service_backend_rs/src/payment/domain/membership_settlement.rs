use chrono::{DateTime, Utc};

use crate::auth::domain::{
    membership::membership_tier_rank, user::resolve_effective_membership_tier,
};

use super::product_kind::{MembershipExpiry, MembershipGrant, ProductSpecError};

pub struct ResolvedMembershipGrant {
    pub tier: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// New orders are paid grants because they carry a validated rule snapshot. Historical
/// orders retain their separate settlement path and its original paid-tier predicate.
pub fn resolve_snapshot_membership_settlement(
    current_tier: &str,
    current_expires_at: Option<DateTime<Utc>>,
    grant: &MembershipGrant,
    now: DateTime<Utc>,
) -> Result<ResolvedMembershipGrant, ProductSpecError> {
    grant.validate()?;
    let current_effective =
        resolve_effective_membership_tier(current_tier, current_expires_at, now);
    let current_rank = membership_tier_rank(&current_effective);
    let paid_rank = membership_tier_rank(&grant.tier);
    let grant_expires_at = grant.expiry.expires_at(now)?;
    // A user can earn perpetual referral benefits while a day-card order is in
    // flight. Preserve them rather than attach an expiry that later falls to V3.
    let preserves_perpetual_membership =
        matches!(grant.expiry, MembershipExpiry::EndOfNextDay { .. })
            && current_rank > membership_tier_rank("V3")
            && current_expires_at.is_none();
    if paid_rank < current_rank || preserves_perpetual_membership {
        return Ok(ResolvedMembershipGrant {
            tier: current_effective,
            expires_at: current_expires_at,
        });
    }
    let expires_at = match grant.expiry {
        MembershipExpiry::Months { .. } => Some(grant_expires_at),
        MembershipExpiry::EndOfNextDay { .. } => match current_expires_at {
            Some(existing) => Some(existing.max(grant_expires_at)),
            None if paid_rank <= current_rank => None,
            None => Some(grant_expires_at),
        },
    };
    Ok(ResolvedMembershipGrant {
        tier: grant.tier.clone(),
        expires_at,
    })
}

#[cfg(test)]
mod tests {
    use super::resolve_snapshot_membership_settlement;
    use crate::payment::domain::product_kind::{MembershipExpiry, MembershipGrant};
    use chrono::{TimeZone, Utc};

    #[test]
    fn snapshot_membership_upgrades_every_paid_tier_including_v4_and_v5() {
        let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
        for tier in ["V3", "V4", "V5", "V6", "V7", "V8", "V9"] {
            let grant = MembershipGrant {
                tier: tier.into(),
                expiry: MembershipExpiry::Months { months: 12 },
            };
            let result = resolve_snapshot_membership_settlement("V1", None, &grant, now).unwrap();
            assert_eq!(result.tier, tier);
            assert_eq!(
                result.expires_at,
                Some(Utc.with_ymd_and_hms(2027, 10, 16, 12, 0, 0).unwrap())
            );
        }
    }

    #[test]
    fn monthly_grant_matches_legacy_refresh_and_does_not_downgrade_higher_membership() {
        let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
        let existing = Some(Utc.with_ymd_and_hms(2028, 1, 1, 0, 0, 0).unwrap());
        for tier in ["V6", "V7", "V8", "V9"] {
            let grant = MembershipGrant {
                tier: tier.into(),
                expiry: MembershipExpiry::Months { months: 12 },
            };
            let result =
                resolve_snapshot_membership_settlement(tier, existing, &grant, now).unwrap();
            assert_eq!(result.tier, tier);
            assert_eq!(
                result.expires_at,
                Some(Utc.with_ymd_and_hms(2027, 10, 16, 12, 0, 0).unwrap()),
                "monthly renewal retains legacy NOW + duration semantics"
            );
            if tier != "V9" {
                let result =
                    resolve_snapshot_membership_settlement("V9", existing, &grant, now).unwrap();
                assert_eq!(result.tier, "V9");
                assert_eq!(result.expires_at, existing);
            }
        }
    }

    #[test]
    fn day_card_renews_without_shortening_or_downgrading() {
        let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
        let grant = MembershipGrant {
            tier: "V4".into(),
            expiry: MembershipExpiry::EndOfNextDay {
                tz: "Asia/Shanghai".into(),
            },
        };
        let next_day_end = Some(Utc.with_ymd_and_hms(2026, 10, 17, 15, 59, 59).unwrap());
        let result = resolve_snapshot_membership_settlement("V3", None, &grant, now).unwrap();
        assert_eq!(result.tier, "V4");
        assert_eq!(result.expires_at, next_day_end);
        let result = resolve_snapshot_membership_settlement(
            "V4",
            Some(now + chrono::Duration::hours(1)),
            &grant,
            now,
        )
        .unwrap();
        assert_eq!(result.expires_at, next_day_end);
        let later = Some(now + chrono::Duration::days(30));
        let result = resolve_snapshot_membership_settlement("V4", later, &grant, now).unwrap();
        assert_eq!(result.expires_at, later);
        let result = resolve_snapshot_membership_settlement("V9", later, &grant, now).unwrap();
        assert_eq!(result.tier, "V9");
        assert_eq!(result.expires_at, later);
        let result = resolve_snapshot_membership_settlement("V4", None, &grant, now).unwrap();
        assert_eq!(
            result.expires_at, None,
            "defensive settlement preserves perpetual referral membership"
        );
    }

    #[test]
    fn expired_membership_is_compared_at_its_effective_v3_tier() {
        let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
        let grant = MembershipGrant {
            tier: "V5".into(),
            expiry: MembershipExpiry::EndOfNextDay {
                tz: "Asia/Shanghai".into(),
            },
        };
        let result = resolve_snapshot_membership_settlement("V9", Some(now), &grant, now).unwrap();
        assert_eq!(result.tier, "V5");
        assert!(result.expires_at.unwrap() > now);
    }

    #[test]
    fn queued_higher_day_card_preserves_perpetual_referral_membership() {
        let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
        let grant = MembershipGrant {
            tier: "V5".into(),
            expiry: MembershipExpiry::EndOfNextDay {
                tz: "Asia/Shanghai".into(),
            },
        };
        let result = resolve_snapshot_membership_settlement("V4", None, &grant, now).unwrap();
        assert_eq!(result.tier, "V4");
        assert_eq!(result.expires_at, None);
    }

    #[test]
    fn invalid_snapshot_grant_cannot_upgrade_a_user() {
        let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
        for grant in [
            MembershipGrant {
                tier: "V100".into(),
                expiry: MembershipExpiry::Months { months: 12 },
            },
            MembershipGrant {
                tier: "V4".into(),
                expiry: MembershipExpiry::Months { months: 0 },
            },
        ] {
            assert!(resolve_snapshot_membership_settlement("V3", None, &grant, now).is_err());
        }
    }
}
