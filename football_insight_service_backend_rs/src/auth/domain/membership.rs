pub use crate::system_config::domain::public_system_config::MembershipTierRuleConfig as MembershipTierRule;

pub fn resolve_referrer_membership_tier(referral_count: i64) -> &'static str {
    match referral_count {
        count if count >= 80 => "V9",
        count if count >= 50 => "V8",
        count if count >= 30 => "V7",
        count if count >= 20 => "V6",
        count if count >= 15 => "V5",
        count if count >= 10 => "V4",
        count if count >= 4 => "V3",
        count if count >= 2 => "V2",
        _ => "V1",
    }
}

pub fn resolve_referrer_membership_tier_from_rules<'a>(
    referral_count: i64,
    rules: &'a [MembershipTierRule],
) -> &'a str {
    rules
        .iter()
        .filter_map(|item| {
            item.min_referrals
                .filter(|threshold| referral_count >= i64::from(*threshold))
                .map(|threshold| (threshold, item.code.as_str()))
        })
        .max_by_key(|(threshold, _)| *threshold)
        .map(|(_, code)| code)
        .unwrap_or_else(|| resolve_referrer_membership_tier(referral_count))
}

pub fn resolve_ticket_watch_poll_interval_seconds(tier: &str, rules: &[MembershipTierRule]) -> i32 {
    let normalized_tier = tier.trim().to_uppercase();

    rules
        .iter()
        .find(|item| item.code.eq_ignore_ascii_case(&normalized_tier))
        .or_else(|| {
            rules
                .iter()
                .find(|item| item.code.eq_ignore_ascii_case("V1"))
        })
        .map(|item| item.ticket_watch_poll_interval_seconds)
        .unwrap_or(600)
}

pub fn membership_tier_rank(tier: &str) -> i32 {
    tier.trim()
        .strip_prefix('V')
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::{
        MembershipTierRule, resolve_referrer_membership_tier,
        resolve_referrer_membership_tier_from_rules, resolve_ticket_watch_poll_interval_seconds,
    };

    #[test]
    fn keeps_v1_before_first_threshold() {
        assert_eq!(resolve_referrer_membership_tier(0), "V1");
        assert_eq!(resolve_referrer_membership_tier(1), "V1");
    }

    #[test]
    fn upgrades_to_v2_after_2_referrals() {
        assert_eq!(resolve_referrer_membership_tier(2), "V2");
        assert_eq!(resolve_referrer_membership_tier(3), "V2");
    }

    #[test]
    fn upgrades_to_v3_after_4_referrals() {
        assert_eq!(resolve_referrer_membership_tier(4), "V3");
        assert_eq!(resolve_referrer_membership_tier(9), "V3");
    }

    #[test]
    fn upgrades_to_v4_after_10_referrals() {
        assert_eq!(resolve_referrer_membership_tier(10), "V4");
        assert_eq!(resolve_referrer_membership_tier(14), "V4");
    }

    #[test]
    fn upgrades_across_all_referral_thresholds() {
        assert_eq!(resolve_referrer_membership_tier(15), "V5");
        assert_eq!(resolve_referrer_membership_tier(20), "V6");
        assert_eq!(resolve_referrer_membership_tier(30), "V7");
        assert_eq!(resolve_referrer_membership_tier(50), "V8");
        assert_eq!(resolve_referrer_membership_tier(80), "V9");
    }

    #[test]
    fn resolves_referrer_membership_tier_from_backend_rules() {
        let rules = vec![
            MembershipTierRule::new("V3", "invite", Some(0), 300),
            MembershipTierRule::new("V4", "referral", Some(12), 120),
            MembershipTierRule::new("V5", "referral", Some(40), 30),
        ];

        assert_eq!(resolve_referrer_membership_tier_from_rules(0, &rules), "V3");
        assert_eq!(
            resolve_referrer_membership_tier_from_rules(12, &rules),
            "V4"
        );
        assert_eq!(
            resolve_referrer_membership_tier_from_rules(39, &rules),
            "V4"
        );
        assert_eq!(
            resolve_referrer_membership_tier_from_rules(40, &rules),
            "V5"
        );
    }

    #[test]
    fn resolves_ticket_watch_poll_interval_from_backend_rules() {
        let rules = vec![
            MembershipTierRule::new("V1", "standard", None, 900),
            MembershipTierRule::new("V3", "invite", Some(0), 180),
            MembershipTierRule::new("V4", "referral", Some(12), 45),
        ];

        assert_eq!(resolve_ticket_watch_poll_interval_seconds("V4", &rules), 45);
        assert_eq!(
            resolve_ticket_watch_poll_interval_seconds("V3", &rules),
            180
        );
        assert_eq!(
            resolve_ticket_watch_poll_interval_seconds("V1", &rules),
            900
        );
        assert_eq!(
            resolve_ticket_watch_poll_interval_seconds("unknown", &rules),
            900
        );
    }
}
