use chrono::{DateTime, Utc};

use super::match_id_unlock::MATCH_ID_UNLOCK_MINIMUM_TIER;
use crate::auth::domain::{
    membership::membership_tier_rank,
    official_account_match_id_access::OfficialAccountMatchIdAccess,
    user::resolve_effective_membership_tier,
};

pub struct OfficialAccountUser {
    pub membership_tier: String,
    pub membership_expires_at: Option<DateTime<Utc>>,
    pub has_unlock_for_match: bool,
}

pub fn resolve_official_account_access(
    user: Option<&OfficialAccountUser>,
    now: DateTime<Utc>,
) -> OfficialAccountMatchIdAccess {
    let Some(user) = user else {
        return OfficialAccountMatchIdAccess::Unrecognized;
    };
    let tier =
        resolve_effective_membership_tier(&user.membership_tier, user.membership_expires_at, now);
    if membership_tier_rank(&tier) >= membership_tier_rank(MATCH_ID_UNLOCK_MINIMUM_TIER)
        || user.has_unlock_for_match
    {
        OfficialAccountMatchIdAccess::Granted
    } else {
        OfficialAccountMatchIdAccess::NotMember
    }
}

/// An official-account openid is not unique in `f_i_users`: a follower can be linked to several
/// accounts. The reply must not depend on which row happens to come back first, so the follower is
/// granted when ANY linked active account qualifies, and recognized (but not a member) otherwise.
pub fn resolve_official_account_access_for_users(
    users: &[OfficialAccountUser],
    now: DateTime<Utc>,
) -> OfficialAccountMatchIdAccess {
    if users.is_empty() {
        return OfficialAccountMatchIdAccess::Unrecognized;
    }
    if users.iter().any(|user| {
        resolve_official_account_access(Some(user), now) == OfficialAccountMatchIdAccess::Granted
    }) {
        OfficialAccountMatchIdAccess::Granted
    } else {
        OfficialAccountMatchIdAccess::NotMember
    }
}
