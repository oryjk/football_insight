/// 单场解锁一口价：500 分 = ¥5.00。
pub const MATCH_ID_UNLOCK_PRICE_CENTS: i32 = 500;

/// 免费查看所需的最低会员等级。
pub const MATCH_ID_UNLOCK_MINIMUM_TIER: &str = "V6";

/// 支付下单时的商品描述。
pub const MATCH_ID_UNLOCK_ORDER_DESCRIPTION: &str = "解锁比赛ID";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchIdUnlockSource {
    Membership,
    Purchase,
}

impl MatchIdUnlockSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Membership => "membership",
            Self::Purchase => "purchase",
        }
    }
}

#[derive(Debug)]
pub enum MatchIdUnlockError {
    MatchNotFound,
    MembershipTierSufficient,
    AlreadyUnlocked,
    WechatBindingRequired,
}

impl std::fmt::Display for MatchIdUnlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MatchNotFound => "比赛不存在",
            Self::MembershipTierSufficient => "V6 及以上会员可直接查看，无需购买",
            Self::AlreadyUnlocked => "本场比赛 ID 已解锁，无需重复购买",
            Self::WechatBindingRequired => "请先绑定微信后再支付",
        };

        write!(f, "{message}")
    }
}

impl std::error::Error for MatchIdUnlockError {}
