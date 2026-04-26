use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportTeamSummary {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_storage_url: Option<String>,
    pub rank_no: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportMatchTeam {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_storage_url: Option<String>,
    pub score: String,
    pub support_count: i64,
    pub support_share_pct: i32,
    pub season_support_rank: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportViewerState {
    pub user_id: Option<Uuid>,
    pub favorite_team_id: Option<i64>,
    pub supported_team_id: Option<i64>,
    pub has_supported: bool,
    pub can_support: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportMatchDetail {
    pub match_id: i64,
    pub season: i32,
    pub round_number: i32,
    pub match_date: String,
    pub match_time: String,
    pub status: String,
    pub kickoff_at: DateTime<Utc>,
    pub support_window_status: SupportWindowStatus,
    pub countdown_seconds: i64,
    pub total_support_count: i64,
    pub home_team: SupportMatchTeam,
    pub away_team: SupportMatchTeam,
    pub viewer: SupportViewerState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportProfileView {
    pub favorite_team: Option<SupportTeamSummary>,
    pub next_match: Option<SupportMatchDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportUserContext {
    pub user_id: Uuid,
    pub favorite_team: Option<SupportTeamSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportWindowStatus {
    Locked,
    Open,
    Closed,
}

impl SupportWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Locked => "locked",
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }
}

pub fn resolve_support_window_status(
    kickoff_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> SupportWindowStatus {
    if now >= kickoff_at {
        return SupportWindowStatus::Closed;
    }

    SupportWindowStatus::Open
}

pub fn validate_vote_team(
    favorite_team_id: Option<i64>,
    supported_team_id: i64,
) -> anyhow::Result<()> {
    let favorite_team_id =
        favorite_team_id.ok_or_else(|| anyhow::anyhow!("favorite team is required"))?;

    if favorite_team_id != supported_team_id {
        anyhow::bail!("supported team must match favorite team");
    }

    Ok(())
}

pub fn validate_supported_team_in_match(
    match_detail: &SupportMatchDetail,
    supported_team_id: i64,
) -> anyhow::Result<()> {
    if match_detail.home_team.team_id == supported_team_id
        || match_detail.away_team.team_id == supported_team_id
    {
        return Ok(());
    }

    anyhow::bail!("supported team does not belong to this match");
}

pub fn validate_support_window(status: SupportWindowStatus) -> anyhow::Result<()> {
    if status == SupportWindowStatus::Open {
        return Ok(());
    }

    anyhow::bail!("support window is not open");
}

pub fn resolve_countdown_seconds(kickoff_at: DateTime<Utc>, now: DateTime<Utc>) -> i64 {
    (kickoff_at - now).num_seconds().max(0)
}

pub fn refresh_match_detail(match_detail: &mut SupportMatchDetail, now: DateTime<Utc>) {
    match_detail.support_window_status =
        resolve_support_window_status(match_detail.kickoff_at, now);
    match_detail.countdown_seconds = resolve_countdown_seconds(match_detail.kickoff_at, now);
    match_detail.viewer.can_support = match_detail.support_window_status
        == SupportWindowStatus::Open
        && match_detail.viewer.favorite_team_id.is_some()
        && !match_detail.viewer.has_supported;
}

pub fn build_kickoff_at(match_date: &str, match_time: &str) -> anyhow::Result<DateTime<Utc>> {
    let date = NaiveDate::parse_from_str(match_date, "%Y-%m-%d")?;
    let time = NaiveTime::parse_from_str(match_time, "%H:%M")?;
    let local = date.and_time(time);
    let offset = FixedOffset::east_opt(8 * 60 * 60).expect("cst offset");

    offset
        .from_local_datetime(&local)
        .single()
        .map(|value| value.with_timezone(&Utc))
        .ok_or_else(|| anyhow::anyhow!("invalid kickoff datetime"))
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    #[test]
    fn support_window_is_open_within_24_hours_before_kickoff() {
        let kickoff_at = Utc
            .with_ymd_and_hms(2026, 4, 12, 11, 35, 0)
            .single()
            .expect("valid kickoff");
        let now = Utc
            .with_ymd_and_hms(2026, 4, 11, 12, 0, 0)
            .single()
            .expect("valid now");

        let status = super::resolve_support_window_status(kickoff_at, now);

        assert_eq!(status.as_str(), "open");
    }

    #[test]
    fn support_window_is_open_any_time_before_kickoff() {
        let kickoff_at = Utc
            .with_ymd_and_hms(2026, 4, 12, 11, 35, 0)
            .single()
            .expect("valid kickoff");
        let now = Utc
            .with_ymd_and_hms(2026, 4, 11, 10, 0, 0)
            .single()
            .expect("valid now");

        let status = super::resolve_support_window_status(kickoff_at, now);

        assert_eq!(status.as_str(), "open");
    }

    #[test]
    fn support_window_is_closed_at_kickoff_and_after() {
        let kickoff_at = Utc
            .with_ymd_and_hms(2026, 4, 12, 11, 35, 0)
            .single()
            .expect("valid kickoff");
        let now = Utc
            .with_ymd_and_hms(2026, 4, 12, 11, 35, 0)
            .single()
            .expect("valid now");

        let status = super::resolve_support_window_status(kickoff_at, now);

        assert_eq!(status.as_str(), "closed");
    }

    #[test]
    fn follower_can_only_support_the_followed_team() {
        super::validate_vote_team(Some(77680), 77680).expect("favorite team should pass");

        let error =
            super::validate_vote_team(Some(77680), 500).expect_err("other team should be rejected");

        assert!(error.to_string().contains("favorite team"));
    }

    #[test]
    fn follower_must_have_selected_a_favorite_team_before_supporting() {
        let error = super::validate_vote_team(None, 77680)
            .expect_err("missing favorite team should be rejected");

        assert!(error.to_string().contains("favorite team"));
    }

    #[test]
    fn kickoff_builder_uses_china_standard_time() {
        let kickoff_at = super::build_kickoff_at("2026-04-12", "19:35").expect("kickoff");

        assert_eq!(kickoff_at.to_rfc3339(), "2026-04-12T11:35:00+00:00");
    }
}
