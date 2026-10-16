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
