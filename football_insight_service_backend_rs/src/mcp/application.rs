use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::Mutex;

use crate::{
    insight::{
        domain::{match_list::MatchListView, overview::MatchOutcome, rankings::RankingsView},
        ports::insight_query_repository::InsightQueryRepository,
    },
    mcp::domain::{
        LeaderboardResult, LeaderboardRow, MatchRow, McpQueryError, Promo, PromoTarget,
        RoundResultsResult, StandingsResult, StandingsRow, match_status_label,
    },
};

const COMPETITION: &str = "中超";
const STANDINGS_SLUG: &str = "standings_with_penalty";
const RECENT_FORM_LEN: usize = 5;
pub const DEFAULT_LEADERBOARD_LIMIT: usize = 20;
pub const MAX_LEADERBOARD_LIMIT: usize = 50;

/// Player leaderboard categories exposed to MCP clients, in display order.
pub const PLAYER_CATEGORIES: [(&str, &str); 8] = [
    ("goals", "进球"),
    ("assists", "助攻"),
    ("shots", "射门"),
    ("shots_on_target", "射正"),
    ("key_passes", "关键传球"),
    ("saves", "扑救"),
    ("yellow_cards", "黄牌"),
    ("red_cards", "红牌"),
];

/// Read-only queries behind the public MCP tools. Responses are cached briefly
/// because the endpoint is anonymous and the underlying queries are not trivial.
pub struct McpQueryService {
    repository: Arc<dyn InsightQueryRepository>,
    public_base_url: String,
    ttl: Duration,
    rankings: Mutex<Option<(Instant, RankingsView)>>,
    live_matches: Mutex<Option<(Instant, MatchListView)>>,
    round_matches: Mutex<HashMap<(i32, i32), (Instant, MatchListView)>>,
    round_rankings: Mutex<HashMap<(i32, i32), (Instant, RankingsView)>>,
}

impl McpQueryService {
    pub fn new(
        repository: Arc<dyn InsightQueryRepository>,
        public_base_url: String,
        ttl: Duration,
    ) -> Self {
        Self {
            repository,
            public_base_url,
            ttl,
            rankings: Mutex::new(None),
            live_matches: Mutex::new(None),
            round_matches: Mutex::new(HashMap::new()),
            round_rankings: Mutex::new(HashMap::new()),
        }
    }

    pub fn promo(&self, target: PromoTarget) -> Promo {
        Promo::for_target(target, &self.public_base_url)
    }

    pub async fn standings(&self, round: Option<i32>) -> Result<StandingsResult, McpQueryError> {
        let live = self.live_rankings().await?;
        let season = live.current_season;
        let rankings = match round {
            None => live,
            Some(round) => {
                self.validate_round(season, round).await?;
                self.round_rankings_cached(season, round).await?
            }
        };
        let table = rankings
            .standings_tables
            .iter()
            .find(|table| table.slug == STANDINGS_SLUG)
            .or_else(|| rankings.standings_tables.first())
            .ok_or_else(|| match round {
                Some(round) => McpQueryError::NotFound(format!(
                    "{season} 赛季第 {round} 轮尚未结束，暂无该轮结束时的积分榜"
                )),
                None => McpQueryError::NotFound("当前赛季暂无积分榜数据".into()),
            })?;
        Ok(StandingsResult {
            kind: "standings",
            competition: COMPETITION,
            season,
            round,
            table: table.label.clone(),
            note: table.note.clone(),
            rows: table
                .entries
                .iter()
                .map(|entry| StandingsRow {
                    rank: entry.rank_no,
                    team: entry.team_name.clone(),
                    played: entry.played,
                    wins: entry.wins,
                    draws: entry.draws,
                    losses: entry.losses,
                    goals_for: entry.goals_for,
                    goals_against: entry.goals_against,
                    goal_difference: entry.goal_difference,
                    points: entry.points,
                    points_adjustment: entry.points_adjustment,
                    recent_form: recent_form(&entry.recent_form),
                })
                .collect(),
            promo: self.promo(PromoTarget::Rankings),
        })
    }

    pub async fn round_results(
        &self,
        round: Option<i32>,
    ) -> Result<RoundResultsResult, McpQueryError> {
        let live = self.live_matches().await?;
        let latest_round = live
            .round_number
            .ok_or_else(|| McpQueryError::NotFound("当前赛季暂无赛程数据".into()))?;
        let season = live.current_season;
        let view = match round {
            None => live,
            Some(round) if round == latest_round => live,
            Some(round) => {
                self.validate_round(season, round).await?;
                self.round_matches_cached(season, round).await?
            }
        };
        let round = view.round_number.unwrap_or(latest_round);
        if view.matches.is_empty() {
            return Err(McpQueryError::NotFound(format!(
                "{season} 赛季第 {round} 轮暂无比赛数据"
            )));
        }
        Ok(RoundResultsResult {
            kind: "round_results",
            competition: COMPETITION,
            season,
            round,
            latest_round,
            matches: view
                .matches
                .iter()
                .map(|card| {
                    let scored = matches!(card.status.as_str(), "finished" | "live")
                        && !card.home_score.trim().is_empty()
                        && !card.away_score.trim().is_empty();
                    MatchRow {
                        date: card.match_date.clone(),
                        time: card.match_time.clone(),
                        status: match_status_label(&card.status).into(),
                        home_team: card.home_team_name.clone(),
                        away_team: card.away_team_name.clone(),
                        home_score: scored.then(|| card.home_score.trim().to_string()),
                        away_score: scored.then(|| card.away_score.trim().to_string()),
                    }
                })
                .collect(),
            promo: self.promo(PromoTarget::Home),
        })
    }

    pub async fn player_leaderboard(
        &self,
        category: &str,
        limit: Option<usize>,
    ) -> Result<LeaderboardResult, McpQueryError> {
        let Some((slug, label)) = PLAYER_CATEGORIES
            .iter()
            .find(|(slug, _)| *slug == category)
            .copied()
        else {
            let allowed: Vec<&str> = PLAYER_CATEGORIES.iter().map(|(slug, _)| *slug).collect();
            return Err(McpQueryError::InvalidArgument(format!(
                "category 只支持：{}",
                allowed.join(", ")
            )));
        };
        let limit = limit.unwrap_or(DEFAULT_LEADERBOARD_LIMIT);
        if limit == 0 || limit > MAX_LEADERBOARD_LIMIT {
            return Err(McpQueryError::InvalidArgument(format!(
                "limit 取值范围 1–{MAX_LEADERBOARD_LIMIT}"
            )));
        }
        let rankings = self.live_rankings().await?;
        let entries = rankings
            .player_categories
            .iter()
            .find(|c| c.slug == slug)
            .map(|c| c.entries.as_slice())
            .unwrap_or_default();
        if entries.is_empty() {
            return Err(McpQueryError::NotFound(format!(
                "当前赛季暂无{label}榜数据"
            )));
        }
        Ok(LeaderboardResult {
            kind: "player_leaderboard",
            competition: COMPETITION,
            season: rankings.current_season,
            category: slug.into(),
            category_label: format!("{label}榜"),
            rows: entries
                .iter()
                .take(limit)
                .map(|entry| LeaderboardRow {
                    rank: entry.rank_no,
                    player: entry.player_name.clone(),
                    team: entry.team_name.clone(),
                    value: entry.score_value.clone(),
                    penalties: entry
                        .penalty_value
                        .clone()
                        .filter(|value| !value.trim().is_empty()),
                })
                .collect(),
            promo: self.promo(PromoTarget::Rankings),
        })
    }

    async fn validate_round(&self, season: i32, round: i32) -> Result<(), McpQueryError> {
        let rounds = self
            .repository
            .list_available_rounds(season)
            .await
            .map_err(unavailable)?;
        let max_round = rounds.iter().map(|r| r.round_number).max().unwrap_or(0);
        if round < 1 || (max_round > 0 && round > max_round) {
            return Err(McpQueryError::InvalidArgument(format!(
                "{season} 赛季中超共 {max_round} 轮，round 取值范围 1–{max_round}"
            )));
        }
        Ok(())
    }

    async fn round_rankings_cached(
        &self,
        season: i32,
        round: i32,
    ) -> Result<RankingsView, McpQueryError> {
        let mut cache = self.round_rankings.lock().await;
        if let Some((at, view)) = cache.get(&(season, round))
            && at.elapsed() < self.ttl
        {
            return Ok(view.clone());
        }
        let view = self
            .repository
            .get_round_rankings(season, round)
            .await
            .map_err(unavailable)?;
        cache.insert((season, round), (Instant::now(), view.clone()));
        Ok(view)
    }

    async fn live_rankings(&self) -> Result<RankingsView, McpQueryError> {
        let mut slot = self.rankings.lock().await;
        if let Some((at, view)) = slot.as_ref()
            && at.elapsed() < self.ttl
        {
            return Ok(view.clone());
        }
        let view = self
            .repository
            .get_live_rankings()
            .await
            .map_err(unavailable)?;
        *slot = Some((Instant::now(), view.clone()));
        Ok(view)
    }

    async fn live_matches(&self) -> Result<MatchListView, McpQueryError> {
        let mut slot = self.live_matches.lock().await;
        if let Some((at, view)) = slot.as_ref()
            && at.elapsed() < self.ttl
        {
            return Ok(view.clone());
        }
        let view = self
            .repository
            .get_live_matches()
            .await
            .map_err(unavailable)?;
        *slot = Some((Instant::now(), view.clone()));
        Ok(view)
    }

    async fn round_matches_cached(
        &self,
        season: i32,
        round: i32,
    ) -> Result<MatchListView, McpQueryError> {
        let mut cache = self.round_matches.lock().await;
        if let Some((at, view)) = cache.get(&(season, round))
            && at.elapsed() < self.ttl
        {
            return Ok(view.clone());
        }
        let view = self
            .repository
            .get_round_matches(season, round)
            .await
            .map_err(unavailable)?;
        cache.insert((season, round), (Instant::now(), view.clone()));
        Ok(view)
    }
}

fn unavailable(error: anyhow::Error) -> McpQueryError {
    tracing::error!(error = %error, "mcp query failed");
    McpQueryError::Unavailable
}

fn recent_form(outcomes: &[MatchOutcome]) -> String {
    let start = outcomes.len().saturating_sub(RECENT_FORM_LEN);
    outcomes[start..]
        .iter()
        .map(|outcome| match outcome {
            MatchOutcome::Win => '胜',
            MatchOutcome::Draw => '平',
            MatchOutcome::Loss => '负',
        })
        .collect()
}

#[cfg(test)]
#[path = "application_tests.rs"]
pub(crate) mod tests;
