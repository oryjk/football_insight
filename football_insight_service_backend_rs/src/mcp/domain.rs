use serde::Serialize;

/// Mini program page a QR code lands on. Scene `s=mcp&c=<slug>` feeds the
/// mini program's acquisition-source tracking (`utils/acquisitionSource.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromoTarget {
    Rankings,
    Home,
}

impl PromoTarget {
    pub const ALL: [PromoTarget; 2] = [PromoTarget::Rankings, PromoTarget::Home];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Rankings => "rankings",
            Self::Home => "home",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|target| target.slug() == slug)
    }

    pub fn page(self) -> &'static str {
        match self {
            Self::Rankings => "pages/rankings/index",
            Self::Home => "pages/home/index",
        }
    }

    pub fn scene(self) -> String {
        format!("s=mcp&c={}", self.slug())
    }

    pub fn call_to_action(self) -> &'static str {
        match self {
            Self::Rankings => "微信扫码，打开「足球洞察」小程序看完整榜单",
            Self::Home => "微信扫码，打开「足球洞察」小程序看比赛详情",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Promo {
    pub mini_program: &'static str,
    pub call_to_action: &'static str,
    pub landing_url: String,
    pub code_image_url: String,
}

impl Promo {
    pub fn for_target(target: PromoTarget, public_base_url: &str) -> Self {
        let base = public_base_url.trim_end_matches('/');
        Self {
            mini_program: "足球洞察",
            call_to_action: target.call_to_action(),
            landing_url: format!("{base}/api/v1/mcp/promo/{}", target.slug()),
            code_image_url: format!("{base}/api/v1/mcp/promo/{}/code", target.slug()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StandingsRow {
    pub rank: i32,
    pub team: String,
    pub played: i32,
    pub wins: i32,
    pub draws: i32,
    pub losses: i32,
    pub goals_for: i32,
    pub goals_against: i32,
    pub goal_difference: i32,
    pub points: i32,
    /// Negative when the team was docked points.
    pub points_adjustment: i32,
    /// Last five results, oldest to newest, e.g. "胜胜平负胜".
    pub recent_form: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StandingsResult {
    pub kind: &'static str,
    pub competition: &'static str,
    pub season: i32,
    /// `None` for the current table, `Some(n)` for the table as round n ended.
    pub round: Option<i32>,
    pub table: String,
    pub note: String,
    pub rows: Vec<StandingsRow>,
    pub promo: Promo,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MatchRow {
    pub date: String,
    pub time: String,
    pub status: String,
    pub home_team: String,
    pub away_team: String,
    /// `None` unless the match has a score (finished or in progress).
    pub home_score: Option<String>,
    pub away_score: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RoundResultsResult {
    pub kind: &'static str,
    pub competition: &'static str,
    pub season: i32,
    pub round: i32,
    pub latest_round: i32,
    pub matches: Vec<MatchRow>,
    pub promo: Promo,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LeaderboardRow {
    pub rank: i32,
    pub player: String,
    pub team: String,
    pub value: String,
    /// Penalty goals included in `value` (goals leaderboard only).
    pub penalties: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LeaderboardResult {
    pub kind: &'static str,
    pub competition: &'static str,
    pub season: i32,
    pub category: String,
    pub category_label: String,
    pub rows: Vec<LeaderboardRow>,
    pub promo: Promo,
}

/// Errors a tool reports back to the model as `isError` results.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum McpQueryError {
    #[error("{0}")]
    InvalidArgument(String),
    #[error("{0}")]
    NotFound(String),
    #[error("数据暂时不可用，请稍后再试")]
    Unavailable,
}

pub fn match_status_label(status: &str) -> &'static str {
    match status {
        "finished" => "已结束",
        "live" => "进行中",
        "postponed" => "延期",
        _ => "未开赛",
    }
}
