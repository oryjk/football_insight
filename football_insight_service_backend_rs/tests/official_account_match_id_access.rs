use chrono::{Duration, TimeZone, Utc};
use football_insight_service_backend_rs::{
    auth::domain::official_account_match_id_access::OfficialAccountMatchIdAccess,
    match_id_unlock::domain::official_account_access::{
        OfficialAccountUser, resolve_official_account_access,
    },
};

#[test]
fn official_account_access_reuses_effective_membership_and_match_unlock_rules() {
    use OfficialAccountMatchIdAccess::{Granted, NotMember, Unrecognized};
    let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
    let cases = [
        ("V1", None, false, NotMember),
        ("V3", None, false, NotMember),
        ("V4", None, false, NotMember),
        ("V5", None, false, NotMember),
        ("V6", None, false, Granted),
        ("V7", None, false, Granted),
        ("V8", None, false, Granted),
        ("V9", None, false, Granted),
        ("V9", Some(now - Duration::seconds(1)), false, NotMember),
        ("V6", Some(now), false, NotMember),
        ("V6", Some(now + Duration::seconds(1)), false, Granted),
        ("V3", None, true, Granted),
        ("V9", Some(now - Duration::seconds(1)), true, Granted),
        ("invalid", None, false, NotMember),
    ];
    for (tier, expires, unlocked, expected) in cases {
        let user = OfficialAccountUser {
            membership_tier: tier.to_owned(),
            membership_expires_at: expires,
            has_unlock_for_match: unlocked,
        };
        assert_eq!(
            resolve_official_account_access(Some(&user), now),
            expected,
            "{tier} {expires:?} {unlocked}"
        );
    }
    assert_eq!(resolve_official_account_access(None, now), Unrecognized);
}

#[test]
fn a_shared_official_openid_is_granted_when_any_linked_account_qualifies() {
    use football_insight_service_backend_rs::match_id_unlock::domain::official_account_access::resolve_official_account_access_for_users;
    use OfficialAccountMatchIdAccess::{Granted, NotMember, Unrecognized};
    let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
    let user = |tier: &str, unlocked: bool| OfficialAccountUser {
        membership_tier: tier.to_owned(),
        membership_expires_at: None,
        has_unlock_for_match: unlocked,
    };
    // Order must not matter: a V9 account and a V1 duplicate behind the same openid.
    assert_eq!(resolve_official_account_access_for_users(&[user("V1", false), user("V9", false)], now), Granted);
    assert_eq!(resolve_official_account_access_for_users(&[user("V9", false), user("V1", false)], now), Granted);
    // A purchased unlock on any linked account also grants.
    assert_eq!(resolve_official_account_access_for_users(&[user("V1", false), user("V3", true)], now), Granted);
    // Only non-qualifying accounts: recognized, but not a member.
    assert_eq!(resolve_official_account_access_for_users(&[user("V1", false), user("V5", false)], now), NotMember);
    // No linked account at all.
    assert_eq!(resolve_official_account_access_for_users(&[], now), Unrecognized);
}
