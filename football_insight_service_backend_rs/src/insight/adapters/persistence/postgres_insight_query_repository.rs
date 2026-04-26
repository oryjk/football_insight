use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone, Utc};
use sqlx::{PgPool, types::Json};

use crate::insight::{
    domain::{
        insight_summary::{RankingMovement, generate_insight_summary},
        match_list::{MatchCard, MatchListView, MatchTechnicalStat},
        overview::{InsightOverview, OverviewMatch, OverviewPlayer, OverviewStanding},
        rankings::{
            PlayerRankingCategory, PlayerRankingEntry, RankingsView, StandingsTable,
            StandingsTableEntry, TeamRankingCategory, TeamRankingEntry,
        },
        round_reference::RoundReference,
        team_insight::{
            AssistContribution, OpponentContribution, PlayerContribution, TeamInsight,
            TeamInsightTeam, TeamInsightsView,
        },
    },
    ports::insight_query_repository::InsightQueryRepository,
};

#[derive(Clone)]
pub struct PostgresInsightQueryRepository {
    pool: PgPool,
}

impl PostgresInsightQueryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn keep_scorers_with_boundary_ties(
        rows: Vec<OverviewPlayerRow>,
        base_limit: usize,
    ) -> Vec<OverviewPlayerRow> {
        if rows.len() <= base_limit {
            return rows;
        }

        let Some(boundary_score) = rows
            .get(base_limit - 1)
            .and_then(|row| row.score_value.parse::<i32>().ok())
        else {
            return rows.into_iter().take(base_limit).collect();
        };

        rows.into_iter()
            .take_while(|row| {
                row.rank_no <= base_limit as i32
                    || row
                        .score_value
                        .parse::<i32>()
                        .ok()
                        .is_some_and(|score| score == boundary_score)
            })
            .collect()
    }

    fn empty_overview(
        current_season: i32,
        view_kind: &str,
        round_number: Option<i32>,
    ) -> InsightOverview {
        InsightOverview {
            view_kind: view_kind.to_string(),
            round_number,
            current_season,
            latest_scrape_finished_at: None,
            total_matches: 0,
            total_teams: 0,
            total_players: 0,
            player_ranking_categories: 0,
            team_ranking_categories: 0,
            standings_top: Vec::new(),
            recent_matches: Vec::new(),
            top_scorers: Vec::new(),
            insight_summary: None,
        }
    }

    async fn fetch_overview(
        &self,
        season: i32,
        view_kind: &str,
        round_number: Option<i32>,
        latest_scrape_finished_at: Option<DateTime<Utc>>,
        standings_snapshot_kind: &str,
        standings_round_number: Option<i32>,
        standings_min_snapshot_at: Option<DateTime<Utc>>,
        ranking_snapshot_kind: &str,
        ranking_round_number: Option<i32>,
        ranking_min_snapshot_at: Option<DateTime<Utc>>,
        matches_round_number: Option<i32>,
    ) -> anyhow::Result<InsightOverview> {
        let total_matches =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM f_i_matches WHERE season = $1")
                .bind(season)
                .fetch_one(&self.pool)
                .await?;

        let total_teams = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM f_i_teams")
            .fetch_one(&self.pool)
            .await?;

        let total_players = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM f_i_players")
            .fetch_one(&self.pool)
            .await?;

        let player_ranking_categories = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM f_i_ranking_categories WHERE scope = 'player'",
        )
        .fetch_one(&self.pool)
        .await?;

        let team_ranking_categories = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM f_i_ranking_categories WHERE scope = 'team'",
        )
        .fetch_one(&self.pool)
        .await?;

        let standings_top = sqlx::query_as::<_, OverviewStandingRow>(
            r#"
            WITH target_snapshot AS (
                SELECT MAX(snapshot_at) AS snapshot_at
                  FROM f_i_standings
                 WHERE season = $1
                   AND snapshot_kind = $2
                   AND ($3::int IS NULL OR round_number = $3)
                   AND ($4::timestamptz IS NULL OR snapshot_at >= $4)
            )
            SELECT s.rank_no, s.team_id, s.team_name, s.points, t.avatar_storage_url
              FROM f_i_standings s
              JOIN target_snapshot ts ON ts.snapshot_at = s.snapshot_at
              LEFT JOIN f_i_teams t ON t.team_id = s.team_id
             WHERE s.season = $1
               AND s.snapshot_kind = $2
               AND ($3::int IS NULL OR s.round_number = $3)
             ORDER BY s.rank_no ASC
             LIMIT 4
            "#,
        )
        .bind(season)
        .bind(standings_snapshot_kind)
        .bind(standings_round_number)
        .bind(standings_min_snapshot_at)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| OverviewStanding {
            rank_no: row.rank_no,
            team_id: row.team_id,
            team_name: row.team_name,
            points: row.points,
            avatar_storage_url: row.avatar_storage_url,
        })
        .collect();

        let recent_matches = sqlx::query_as::<_, OverviewMatchRow>(
            r#"
            SELECT
                match_id,
                round_number,
                match_date::text AS match_date,
                match_time,
                home_team_name,
                away_team_name,
                home_score,
                away_score
             FROM f_i_matches
             WHERE season = $1
               AND status = '3'
               AND ($2::int IS NULL OR round_number = $2)
             ORDER BY round_number DESC, match_date DESC, match_time DESC
             LIMIT 3
            "#,
        )
        .bind(season)
        .bind(matches_round_number)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| OverviewMatch {
            match_id: row.match_id,
            round_number: row.round_number,
            match_date: row.match_date,
            match_time: row.match_time,
            home_team_name: row.home_team_name,
            away_team_name: row.away_team_name,
            home_score: row.home_score,
            away_score: row.away_score,
        })
        .collect();

        let top_scorer_rows = sqlx::query_as::<_, OverviewPlayerRow>(
            r#"
            WITH latest_goal_snapshot AS (
                SELECT s.id
                  FROM f_i_player_ranking_snapshots s
                  JOIN f_i_ranking_categories c ON c.id = s.category_id
                 WHERE s.season = $1
                   AND s.snapshot_kind = $2
                   AND ($3::int IS NULL OR s.round_number = $3)
                   AND ($4::timestamptz IS NULL OR s.snapshot_at >= $4)
                   AND c.scope = 'player'
                   AND c.slug = 'goals'
                 ORDER BY s.snapshot_at DESC, s.id DESC
                 LIMIT 1
            )
            SELECT
                e.rank_no,
                e.player_id,
                e.player_name,
                e.team_name,
                e.score_value,
                p.avatar_storage_url
              FROM latest_goal_snapshot s
              JOIN f_i_player_ranking_entries e ON e.snapshot_id = s.id
              LEFT JOIN f_i_players p ON p.player_id = e.player_id
             ORDER BY e.rank_no ASC
            "#,
        )
        .bind(season)
        .bind(ranking_snapshot_kind)
        .bind(ranking_round_number)
        .bind(ranking_min_snapshot_at)
        .fetch_all(&self.pool)
        .await?;

        let top_scorers = Self::keep_scorers_with_boundary_ties(top_scorer_rows, 5)
            .into_iter()
            .map(|row| OverviewPlayer {
                rank_no: row.rank_no,
                player_id: row.player_id,
                player_name: row.player_name,
                team_name: row.team_name,
                score_value: row.score_value,
                avatar_storage_url: row.avatar_storage_url,
            })
            .collect();

        Ok(InsightOverview {
            view_kind: view_kind.to_string(),
            round_number,
            current_season: season,
            latest_scrape_finished_at,
            total_matches,
            total_teams,
            total_players,
            player_ranking_categories,
            team_ranking_categories,
            standings_top,
            recent_matches,
            top_scorers,
            insight_summary: None,
        })
    }

    fn empty_rankings(
        current_season: i32,
        view_kind: &str,
        round_number: Option<i32>,
    ) -> RankingsView {
        RankingsView {
            view_kind: view_kind.to_string(),
            round_number,
            current_season,
            standings_tables: Vec::new(),
            team_categories: Vec::new(),
            player_categories: Vec::new(),
        }
    }

    fn empty_match_list(
        current_season: i32,
        view_kind: &str,
        round_number: Option<i32>,
    ) -> MatchListView {
        MatchListView {
            view_kind: view_kind.to_string(),
            round_number,
            current_season,
            matches: Vec::new(),
        }
    }

    fn empty_team_insights(
        current_season: i32,
        view_kind: &str,
        round_number: Option<i32>,
    ) -> TeamInsightsView {
        TeamInsightsView {
            view_kind: view_kind.to_string(),
            round_number,
            current_season,
            teams: Vec::new(),
            insights: Vec::new(),
        }
    }

    fn map_match_status(status: &str) -> &'static str {
        match status {
            "1" => "scheduled",
            "3" => "finished",
            _ => "live",
        }
    }

    fn china_local_naive_to_utc(value: NaiveDateTime) -> Option<DateTime<Utc>> {
        let offset = FixedOffset::east_opt(8 * 60 * 60).expect("china offset");
        offset
            .from_local_datetime(&value)
            .single()
            .map(|datetime| datetime.with_timezone(&Utc))
    }

    fn summarize_rounds(season: i32, rows: Vec<RoundProgressRow>) -> Vec<RoundReference> {
        let current_round_number = rows
            .iter()
            .find(|row| row.completed_matches < row.total_matches)
            .map(|row| row.round_number);

        rows.into_iter()
            .map(|row| {
                let status = if row.completed_matches >= row.total_matches {
                    "completed"
                } else if current_round_number == Some(row.round_number) {
                    "current"
                } else {
                    "upcoming"
                };

                RoundReference {
                    season,
                    round_number: row.round_number,
                    finalized_at: row.finalized_at.map(|item| item.to_rfc3339()),
                    status: status.to_string(),
                    total_matches: row.total_matches as i32,
                    completed_matches: row.completed_matches as i32,
                }
            })
            .collect()
    }

    fn resolve_live_match_round_number(rounds: &[RoundReference]) -> Option<(i32, bool)> {
        rounds
            .iter()
            .find(|round| round.status == "current")
            .map(|round| (round.round_number, false))
            .or_else(|| {
                rounds
                    .iter()
                    .rev()
                    .find(|round| round.status == "completed")
                    .map(|round| (round.round_number, true))
            })
    }

    fn prepend_standings_category(
        mut team_categories: Vec<TeamRankingCategory>,
        standings_rows: Vec<OverviewStandingRow>,
    ) -> Vec<TeamRankingCategory> {
        if standings_rows.is_empty() {
            return team_categories;
        }

        let standings_category = TeamRankingCategory {
            slug: "standings".to_string(),
            label: "球队排名".to_string(),
            item_id: 0,
            entries: standings_rows
                .into_iter()
                .map(|row| TeamRankingEntry {
                    rank_no: row.rank_no,
                    team_id: row.team_id,
                    team_name: row.team_name,
                    score_value: row.points.to_string(),
                    avatar_storage_url: row.avatar_storage_url,
                })
                .collect(),
        };

        team_categories.insert(0, standings_category);
        team_categories
    }

    fn build_standings_tables(rows: &[StandingsTableRow]) -> Vec<StandingsTable> {
        if rows.is_empty() {
            return Vec::new();
        }

        let to_entry = |row: &StandingsTableRow| StandingsTableEntry {
            rank_no: row.rank_no,
            team_id: row.team_id,
            team_name: row.team_name.clone(),
            played: row.played,
            wins: row.wins,
            draws: row.draws,
            losses: row.losses,
            goals_for: row.goals_for,
            goals_against: row.goals_against,
            goal_difference: row.goal_difference,
            points: row.points,
            points_without_penalty: row.wins * 3 + row.draws,
            points_adjustment: row.points - (row.wins * 3 + row.draws),
            avatar_storage_url: row.avatar_storage_url.clone(),
        };

        let with_penalty_entries = rows.iter().map(to_entry).collect();

        let mut without_penalty_entries: Vec<StandingsTableEntry> =
            rows.iter().map(to_entry).collect();
        without_penalty_entries.sort_by(|left, right| {
            right
                .points_without_penalty
                .cmp(&left.points_without_penalty)
                .then_with(|| right.goal_difference.cmp(&left.goal_difference))
                .then_with(|| right.goals_for.cmp(&left.goals_for))
                .then_with(|| left.rank_no.cmp(&right.rank_no))
        });

        for (index, entry) in without_penalty_entries.iter_mut().enumerate() {
            entry.rank_no = index as i32 + 1;
        }

        vec![
            StandingsTable {
                slug: "standings_without_penalty".to_string(),
                label: "无罚分版积分榜".to_string(),
                note: "按胜平负推导的理论积分排序，不计入罚分调整。".to_string(),
                entries: without_penalty_entries,
            },
            StandingsTable {
                slug: "standings_with_penalty".to_string(),
                label: "含罚分版积分榜".to_string(),
                note: "按当前实际积分排序，已包含罚分影响。".to_string(),
                entries: with_penalty_entries,
            },
        ]
    }

    async fn fetch_rankings(
        &self,
        season: i32,
        view_kind: &str,
        round_number: Option<i32>,
        snapshot_kind: &str,
        minimum_snapshot_at: Option<DateTime<Utc>>,
    ) -> anyhow::Result<RankingsView> {
        let team_rows = sqlx::query_as::<_, TeamRankingRow>(
            r#"
            WITH latest_snapshots AS (
                SELECT DISTINCT ON (c.item_id)
                    s.id,
                    c.item_id,
                    c.slug,
                    c.label
                FROM f_i_team_ranking_snapshots s
                JOIN f_i_ranking_categories c ON c.id = s.category_id
                WHERE s.season = $1
                  AND s.snapshot_kind = $2
                  AND c.scope = 'team'
                  AND ($3::int IS NULL OR s.round_number = $3)
                  AND ($4::timestamptz IS NULL OR s.snapshot_at >= $4)
                ORDER BY c.item_id, s.snapshot_at DESC, s.id DESC
            )
            SELECT
                ls.item_id,
                ls.slug,
                ls.label,
                e.rank_no,
                e.team_id,
                e.team_name,
                e.score_value,
                t.avatar_storage_url
            FROM latest_snapshots ls
            JOIN f_i_team_ranking_entries e ON e.snapshot_id = ls.id
            LEFT JOIN f_i_teams t ON t.team_id = e.team_id
            ORDER BY ls.item_id ASC, e.rank_no ASC
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .bind(round_number)
        .bind(minimum_snapshot_at)
        .fetch_all(&self.pool)
        .await?;

        let standings_table_rows = sqlx::query_as::<_, StandingsTableRow>(
            r#"
            WITH target_snapshot AS (
                SELECT MAX(snapshot_at) AS snapshot_at
                  FROM f_i_standings
                 WHERE season = $1
                   AND snapshot_kind = $2
                   AND ($3::int IS NULL OR round_number = $3)
                   AND ($4::timestamptz IS NULL OR snapshot_at >= $4)
            )
            SELECT
                s.rank_no,
                s.team_id,
                s.team_name,
                s.played,
                s.wins,
                s.draws,
                s.losses,
                s.goals_for,
                s.goals_against,
                s.goal_difference,
                s.points,
                t.avatar_storage_url
              FROM f_i_standings s
              JOIN target_snapshot ts ON ts.snapshot_at = s.snapshot_at
              LEFT JOIN f_i_teams t ON t.team_id = s.team_id
             WHERE s.season = $1
               AND s.snapshot_kind = $2
               AND ($3::int IS NULL OR s.round_number = $3)
             ORDER BY s.rank_no ASC
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .bind(round_number)
        .bind(minimum_snapshot_at)
        .fetch_all(&self.pool)
        .await?;

        let player_rows = sqlx::query_as::<_, PlayerRankingRow>(
            r#"
            WITH latest_snapshots AS (
                SELECT DISTINCT ON (c.item_id)
                    s.id,
                    c.item_id,
                    c.slug,
                    c.label
                FROM f_i_player_ranking_snapshots s
                JOIN f_i_ranking_categories c ON c.id = s.category_id
                WHERE s.season = $1
                  AND s.snapshot_kind = $2
                  AND c.scope = 'player'
                  AND ($3::int IS NULL OR s.round_number = $3)
                  AND ($4::timestamptz IS NULL OR s.snapshot_at >= $4)
                ORDER BY c.item_id, s.snapshot_at DESC, s.id DESC
            )
            SELECT
                ls.item_id,
                ls.slug,
                ls.label,
                e.rank_no,
                e.player_id,
                e.player_name,
                e.team_id,
                e.team_name,
                e.score_value,
                e.penalty_value,
                p.avatar_storage_url
            FROM latest_snapshots ls
            JOIN f_i_player_ranking_entries e ON e.snapshot_id = ls.id
            LEFT JOIN f_i_players p ON p.player_id = e.player_id
            ORDER BY ls.item_id ASC, e.rank_no ASC
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .bind(round_number)
        .bind(minimum_snapshot_at)
        .fetch_all(&self.pool)
        .await?;

        let standings_rows = sqlx::query_as::<_, OverviewStandingRow>(
            r#"
            WITH target_snapshot AS (
                SELECT MAX(snapshot_at) AS snapshot_at
                  FROM f_i_standings
                 WHERE season = $1
                   AND snapshot_kind = $2
                   AND ($3::int IS NULL OR round_number = $3)
                   AND ($4::timestamptz IS NULL OR snapshot_at >= $4)
            )
            SELECT s.rank_no, s.team_id, s.team_name, s.points, t.avatar_storage_url
              FROM f_i_standings s
              JOIN target_snapshot ts ON ts.snapshot_at = s.snapshot_at
              LEFT JOIN f_i_teams t ON t.team_id = s.team_id
             WHERE s.season = $1
               AND s.snapshot_kind = $2
               AND ($3::int IS NULL OR s.round_number = $3)
             ORDER BY s.rank_no ASC
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .bind(round_number)
        .bind(minimum_snapshot_at)
        .fetch_all(&self.pool)
        .await?;

        let mut team_categories: Vec<TeamRankingCategory> = Vec::new();
        for row in team_rows {
            if team_categories
                .last()
                .is_none_or(|category| category.item_id != row.item_id)
            {
                team_categories.push(TeamRankingCategory {
                    slug: row.slug.clone(),
                    label: row.label.clone(),
                    item_id: row.item_id,
                    entries: Vec::new(),
                });
            }

            if let Some(category) = team_categories.last_mut() {
                category.entries.push(TeamRankingEntry {
                    rank_no: row.rank_no,
                    team_id: row.team_id,
                    team_name: row.team_name,
                    score_value: row.score_value,
                    avatar_storage_url: row.avatar_storage_url,
                });
            }
        }

        let team_categories = Self::prepend_standings_category(team_categories, standings_rows);

        let mut player_categories: Vec<PlayerRankingCategory> = Vec::new();
        for row in player_rows {
            if player_categories
                .last()
                .is_none_or(|category| category.item_id != row.item_id)
            {
                player_categories.push(PlayerRankingCategory {
                    slug: row.slug.clone(),
                    label: row.label.clone(),
                    item_id: row.item_id,
                    entries: Vec::new(),
                });
            }

            if let Some(category) = player_categories.last_mut() {
                category.entries.push(PlayerRankingEntry {
                    rank_no: row.rank_no,
                    player_id: row.player_id,
                    player_name: row.player_name,
                    team_id: row.team_id,
                    team_name: row.team_name,
                    score_value: row.score_value,
                    penalty_value: row.penalty_value,
                    avatar_storage_url: row.avatar_storage_url,
                });
            }
        }

        Ok(RankingsView {
            view_kind: view_kind.to_string(),
            round_number,
            current_season: season,
            standings_tables: Self::build_standings_tables(&standings_table_rows),
            team_categories,
            player_categories,
        })
    }

    async fn fetch_matches(
        &self,
        season: i32,
        view_kind: &str,
        round_number: Option<i32>,
        finished_only: bool,
    ) -> anyhow::Result<MatchListView> {
        let rows = if let Some(round_number) = round_number {
            sqlx::query_as::<_, MatchCardRow>(
                r#"
                SELECT
                    m.match_id,
                    m.round_number,
                    m.match_date::text AS match_date,
                    m.match_time,
                    m.status,
                    m.home_team_id,
                    m.home_team_name,
                    m.home_score,
                    m.away_team_id,
                    m.away_team_name,
                    m.away_score,
                    ht.avatar_storage_url AS home_team_avatar,
                    at.avatar_storage_url AS away_team_avatar,
                    m.leisu_match_id,
                    m.home_corners,
                    m.away_corners,
                    m.corner_source,
                    m.technical_stats
                FROM f_i_matches m
                LEFT JOIN f_i_teams ht ON ht.team_id = m.home_team_id
                LEFT JOIN f_i_teams at ON at.team_id = m.away_team_id
                WHERE m.season = $1
                  AND m.round_number = $2
                  AND ($3::bool = false OR m.status = '3')
                ORDER BY m.match_date ASC, m.match_time ASC, m.match_id ASC
                LIMIT 32
                "#,
            )
            .bind(season)
            .bind(round_number)
            .bind(finished_only)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, MatchCardRow>(
                r#"
                SELECT
                    m.match_id,
                    m.round_number,
                    m.match_date::text AS match_date,
                    m.match_time,
                    m.status,
                    m.home_team_id,
                    m.home_team_name,
                    m.home_score,
                    m.away_team_id,
                    m.away_team_name,
                    m.away_score,
                    ht.avatar_storage_url AS home_team_avatar,
                    at.avatar_storage_url AS away_team_avatar,
                    m.leisu_match_id,
                    m.home_corners,
                    m.away_corners,
                    m.corner_source,
                    m.technical_stats
                FROM f_i_matches m
                LEFT JOIN f_i_teams ht ON ht.team_id = m.home_team_id
                LEFT JOIN f_i_teams at ON at.team_id = m.away_team_id
                WHERE m.season = $1
                  AND ($2::bool = false OR m.status = '3')
                ORDER BY m.round_number DESC, m.match_date DESC, m.match_time DESC, m.match_id DESC
                LIMIT 20
                "#,
            )
            .bind(season)
            .bind(finished_only)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(MatchListView {
            view_kind: view_kind.to_string(),
            round_number,
            current_season: season,
            matches: rows
                .into_iter()
                .map(|row| MatchCard {
                    match_id: row.match_id,
                    round_number: row.round_number,
                    match_date: row.match_date,
                    match_time: row.match_time,
                    status: Self::map_match_status(&row.status).to_string(),
                    home_team_id: row.home_team_id,
                    home_team_name: row.home_team_name,
                    home_score: row.home_score,
                    away_team_id: row.away_team_id,
                    away_team_name: row.away_team_name,
                    away_score: row.away_score,
                    home_team_avatar: row.home_team_avatar,
                    away_team_avatar: row.away_team_avatar,
                    leisu_match_id: row.leisu_match_id,
                    home_corners: row.home_corners,
                    away_corners: row.away_corners,
                    corner_source: row.corner_source,
                    technical_stats: row.technical_stats.map(|value| value.0).unwrap_or_default(),
                })
                .collect(),
        })
    }

    async fn fetch_live_team_insights(&self, season: i32) -> anyhow::Result<TeamInsightsView> {
        let rows = sqlx::query_as::<_, TeamInsightRow>(
            r#"
            SELECT
                ti.team_id,
                ti.team_name,
                ti.rank_no,
                COALESCE(ti.avatar_storage_url, t.avatar_storage_url) as avatar_storage_url,
                ti.goals_for_total,
                ti.goals_against_total,
                ti.goals_for_by_opponent,
                ti.goals_for_by_player,
                ti.assists_for_by_player,
                ti.goals_against_by_opponent,
                ti.round_number
              FROM f_i_team_insights ti
              LEFT JOIN f_i_teams t ON t.team_id = ti.team_id
             WHERE ti.season = $1
               AND ti.snapshot_kind = 'live'
             ORDER BY ti.rank_no ASC, ti.team_id ASC
            "#,
        )
        .bind(season)
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Ok(Self::empty_team_insights(season, "live", None));
        }

        let round_number = rows.first().and_then(|item| item.round_number);
        let teams = rows
            .iter()
            .map(|row| TeamInsightTeam {
                team_id: row.team_id,
                team_name: row.team_name.clone(),
                rank_no: row.rank_no,
                avatar_storage_url: row.avatar_storage_url.clone(),
            })
            .collect();
        let insights = rows
            .into_iter()
            .map(|row| TeamInsight {
                team_id: row.team_id,
                team_name: row.team_name,
                rank_no: row.rank_no,
                avatar_storage_url: row.avatar_storage_url,
                goals_for_total: row.goals_for_total,
                goals_against_total: row.goals_against_total,
                goals_for_by_opponent: row.goals_for_by_opponent.0,
                goals_for_by_player: row.goals_for_by_player.0,
                assists_for_by_player: row.assists_for_by_player.0,
                goals_against_by_opponent: row.goals_against_by_opponent.0,
            })
            .collect();

        Ok(TeamInsightsView {
            view_kind: "live".to_string(),
            round_number,
            current_season: season,
            teams,
            insights,
        })
    }

    async fn fetch_round_completion_cutoff(
        &self,
        season: i32,
        round_number: i32,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let cutoff = sqlx::query_scalar::<_, Option<NaiveDateTime>>(
            r#"
            SELECT MAX((match_date::timestamp + match_time::time) + INTERVAL '120 minutes') AS cutoff
              FROM f_i_matches
             WHERE season = $1
               AND round_number = $2
             GROUP BY round_number
            HAVING BOOL_AND(status = '3')
            "#,
        )
        .bind(season)
        .bind(round_number)
        .fetch_optional(&self.pool)
        .await
        .map(|item| item.flatten())?;

        Ok(cutoff.and_then(Self::china_local_naive_to_utc))
    }

    async fn fetch_next_round_first_kickoff(
        &self,
        season: i32,
        round_number: i32,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let kickoff = sqlx::query_scalar::<_, NaiveDateTime>(
            r#"
            SELECT MIN(match_date::timestamp + match_time::time)
              FROM f_i_matches
             WHERE season = $1
               AND round_number > $2
            "#,
        )
        .bind(season)
        .bind(round_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(anyhow::Error::from)?;

        Ok(kickoff.and_then(Self::china_local_naive_to_utc))
    }

    async fn fetch_valid_round_finalized_at(
        &self,
        season: i32,
        round_number: i32,
    ) -> anyhow::Result<Option<DateTime<Utc>>> {
        let Some(round_cutoff) = self
            .fetch_round_completion_cutoff(season, round_number)
            .await?
        else {
            return Ok(None);
        };

        let next_round_start = self
            .fetch_next_round_first_kickoff(season, round_number)
            .await?;

        sqlx::query_scalar::<_, DateTime<Utc>>(
            r#"
            SELECT MAX(snapshot_at)
              FROM f_i_team_ranking_snapshots
             WHERE season = $1
               AND round_number = $2
               AND snapshot_kind = 'round_final'
               AND snapshot_at >= $3
               AND ($4::timestamptz IS NULL OR snapshot_at < $4)
            "#,
        )
        .bind(season)
        .bind(round_number)
        .bind(round_cutoff)
        .bind(next_round_start)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn fetch_standings_movement(
        &self,
        season: i32,
        snapshot_kind: &str,
        round_number: Option<i32>,
    ) -> anyhow::Result<Option<RankingMovement>> {
        let current_rows = self
            .fetch_current_standings_rows(season, snapshot_kind, round_number)
            .await?;
        let previous_rows = self
            .fetch_previous_standings_rows(season, snapshot_kind, round_number)
            .await?;

        Ok(select_best_riser(
            current_rows
                .into_iter()
                .map(|item| (item.team_id, item.team_name, item.rank_no))
                .collect(),
            previous_rows
                .into_iter()
                .map(|item| (item.team_id, item.team_name, item.rank_no))
                .collect(),
        ))
    }

    async fn fetch_scorer_movement(
        &self,
        season: i32,
        snapshot_kind: &str,
        round_number: Option<i32>,
    ) -> anyhow::Result<Option<RankingMovement>> {
        let current_rows = self
            .fetch_current_goal_scorer_rows(season, snapshot_kind, round_number)
            .await?;
        let previous_rows = self
            .fetch_previous_goal_scorer_rows(season, snapshot_kind, round_number)
            .await?;

        Ok(select_best_riser(
            current_rows
                .into_iter()
                .map(|item| (item.player_id, item.player_name, item.rank_no))
                .collect(),
            previous_rows
                .into_iter()
                .map(|item| (item.player_id, item.player_name, item.rank_no))
                .collect(),
        ))
    }

    async fn fetch_current_standings_rows(
        &self,
        season: i32,
        snapshot_kind: &str,
        round_number: Option<i32>,
    ) -> anyhow::Result<Vec<StandingRankRow>> {
        if let Some(round_number) = round_number {
            let latest_snapshot = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
                r#"
                SELECT MAX(snapshot_at)
                  FROM f_i_standings
                 WHERE season = $1
                   AND snapshot_kind = $2
                   AND round_number = $3
                "#,
            )
            .bind(season)
            .bind(snapshot_kind)
            .bind(round_number)
            .fetch_one(&self.pool)
            .await?;

            let Some(latest_snapshot) = latest_snapshot else {
                return Ok(Vec::new());
            };

            return sqlx::query_as::<_, StandingRankRow>(
                r#"
                SELECT team_id, team_name, rank_no
                  FROM f_i_standings
                 WHERE season = $1
                   AND snapshot_kind = $2
                   AND round_number = $3
                   AND snapshot_at = $4
                "#,
            )
            .bind(season)
            .bind(snapshot_kind)
            .bind(round_number)
            .bind(latest_snapshot)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into);
        }

        let latest_snapshot = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            SELECT snapshot_at
              FROM f_i_standings
             WHERE season = $1
               AND snapshot_kind = $2
             GROUP BY snapshot_at
             ORDER BY snapshot_at DESC
             LIMIT 1
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .fetch_one(&self.pool)
        .await?;

        let Some(latest_snapshot) = latest_snapshot else {
            return Ok(Vec::new());
        };

        sqlx::query_as::<_, StandingRankRow>(
            r#"
            SELECT team_id, team_name, rank_no
              FROM f_i_standings
             WHERE season = $1
               AND snapshot_kind = $2
               AND snapshot_at = $3
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .bind(latest_snapshot)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn fetch_previous_standings_rows(
        &self,
        season: i32,
        snapshot_kind: &str,
        round_number: Option<i32>,
    ) -> anyhow::Result<Vec<StandingRankRow>> {
        if let Some(round_number) = round_number {
            let previous_round = sqlx::query_scalar::<_, Option<i32>>(
                r#"
                SELECT MAX(round_number)
                  FROM f_i_standings
                 WHERE season = $1
                   AND snapshot_kind = $2
                   AND round_number < $3
                "#,
            )
            .bind(season)
            .bind(snapshot_kind)
            .bind(round_number)
            .fetch_one(&self.pool)
            .await?;

            let Some(previous_round) = previous_round else {
                return Ok(Vec::new());
            };

            let latest_snapshot = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
                r#"
                SELECT MAX(snapshot_at)
                  FROM f_i_standings
                 WHERE season = $1
                   AND snapshot_kind = $2
                   AND round_number = $3
                "#,
            )
            .bind(season)
            .bind(snapshot_kind)
            .bind(previous_round)
            .fetch_one(&self.pool)
            .await?;

            let Some(latest_snapshot) = latest_snapshot else {
                return Ok(Vec::new());
            };

            return sqlx::query_as::<_, StandingRankRow>(
                r#"
                SELECT team_id, team_name, rank_no
                  FROM f_i_standings
                 WHERE season = $1
                   AND snapshot_kind = $2
                   AND round_number = $3
                   AND snapshot_at = $4
                "#,
            )
            .bind(season)
            .bind(snapshot_kind)
            .bind(previous_round)
            .bind(latest_snapshot)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into);
        }

        let snapshots = sqlx::query_scalar::<_, DateTime<Utc>>(
            r#"
            SELECT snapshot_at
              FROM f_i_standings
             WHERE season = $1
               AND snapshot_kind = $2
             GROUP BY snapshot_at
             ORDER BY snapshot_at DESC
             LIMIT 2
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .fetch_all(&self.pool)
        .await?;

        let Some(previous_snapshot) = snapshots.get(1).cloned() else {
            return Ok(Vec::new());
        };

        sqlx::query_as::<_, StandingRankRow>(
            r#"
            SELECT team_id, team_name, rank_no
              FROM f_i_standings
             WHERE season = $1
               AND snapshot_kind = $2
               AND snapshot_at = $3
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .bind(previous_snapshot)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn fetch_current_goal_scorer_rows(
        &self,
        season: i32,
        snapshot_kind: &str,
        round_number: Option<i32>,
    ) -> anyhow::Result<Vec<PlayerRankRow>> {
        let snapshot_id = self
            .fetch_current_goal_snapshot_id(season, snapshot_kind, round_number)
            .await?;

        let Some(snapshot_id) = snapshot_id else {
            return Ok(Vec::new());
        };

        sqlx::query_as::<_, PlayerRankRow>(
            r#"
            SELECT player_id, player_name, rank_no
              FROM f_i_player_ranking_entries
             WHERE snapshot_id = $1
            "#,
        )
        .bind(snapshot_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn fetch_previous_goal_scorer_rows(
        &self,
        season: i32,
        snapshot_kind: &str,
        round_number: Option<i32>,
    ) -> anyhow::Result<Vec<PlayerRankRow>> {
        let snapshot_id = self
            .fetch_previous_goal_snapshot_id(season, snapshot_kind, round_number)
            .await?;

        let Some(snapshot_id) = snapshot_id else {
            return Ok(Vec::new());
        };

        sqlx::query_as::<_, PlayerRankRow>(
            r#"
            SELECT player_id, player_name, rank_no
              FROM f_i_player_ranking_entries
             WHERE snapshot_id = $1
            "#,
        )
        .bind(snapshot_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn fetch_current_goal_snapshot_id(
        &self,
        season: i32,
        snapshot_kind: &str,
        round_number: Option<i32>,
    ) -> anyhow::Result<Option<i64>> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT s.id
              FROM f_i_player_ranking_snapshots s
              JOIN f_i_ranking_categories c ON c.id = s.category_id
             WHERE s.season = $1
               AND s.snapshot_kind = $2
               AND c.scope = 'player'
               AND c.slug = 'goals'
               AND ($3::int IS NULL OR s.round_number = $3)
             ORDER BY s.snapshot_at DESC, s.id DESC
             LIMIT 1
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .bind(round_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn fetch_previous_goal_snapshot_id(
        &self,
        season: i32,
        snapshot_kind: &str,
        round_number: Option<i32>,
    ) -> anyhow::Result<Option<i64>> {
        if let Some(round_number) = round_number {
            let previous_round = sqlx::query_scalar::<_, Option<i32>>(
                r#"
                SELECT MAX(s.round_number)
                  FROM f_i_player_ranking_snapshots s
                  JOIN f_i_ranking_categories c ON c.id = s.category_id
                 WHERE s.season = $1
                   AND s.snapshot_kind = $2
                   AND c.scope = 'player'
                   AND c.slug = 'goals'
                   AND s.round_number < $3
                "#,
            )
            .bind(season)
            .bind(snapshot_kind)
            .bind(round_number)
            .fetch_one(&self.pool)
            .await?;

            let Some(previous_round) = previous_round else {
                return Ok(None);
            };

            return sqlx::query_scalar::<_, i64>(
                r#"
                SELECT s.id
                  FROM f_i_player_ranking_snapshots s
                  JOIN f_i_ranking_categories c ON c.id = s.category_id
                 WHERE s.season = $1
                   AND s.snapshot_kind = $2
                   AND c.scope = 'player'
                   AND c.slug = 'goals'
                   AND s.round_number = $3
                 ORDER BY s.snapshot_at DESC, s.id DESC
                 LIMIT 1
                "#,
            )
            .bind(season)
            .bind(snapshot_kind)
            .bind(previous_round)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into);
        }

        let snapshot_ids = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT s.id
              FROM f_i_player_ranking_snapshots s
              JOIN f_i_ranking_categories c ON c.id = s.category_id
             WHERE s.season = $1
               AND s.snapshot_kind = $2
               AND c.scope = 'player'
               AND c.slug = 'goals'
             ORDER BY s.snapshot_at DESC, s.id DESC
             LIMIT 2
            "#,
        )
        .bind(season)
        .bind(snapshot_kind)
        .fetch_all(&self.pool)
        .await?;

        Ok(snapshot_ids.get(1).copied())
    }
}

#[derive(sqlx::FromRow)]
struct LatestRunRow {
    season: i32,
    finished_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct OverviewStandingRow {
    rank_no: i32,
    team_id: i64,
    team_name: String,
    points: i32,
    avatar_storage_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct OverviewMatchRow {
    match_id: i64,
    round_number: i32,
    match_date: String,
    match_time: String,
    home_team_name: String,
    away_team_name: String,
    home_score: String,
    away_score: String,
}

#[derive(sqlx::FromRow)]
struct OverviewPlayerRow {
    rank_no: i32,
    player_id: i64,
    player_name: String,
    team_name: String,
    score_value: String,
    avatar_storage_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TeamRankingRow {
    item_id: i32,
    slug: String,
    label: String,
    rank_no: i32,
    team_id: i64,
    team_name: String,
    score_value: String,
    avatar_storage_url: Option<String>,
}

#[derive(sqlx::FromRow, Clone)]
struct StandingsTableRow {
    rank_no: i32,
    team_id: i64,
    team_name: String,
    played: i32,
    wins: i32,
    draws: i32,
    losses: i32,
    goals_for: i32,
    goals_against: i32,
    goal_difference: i32,
    points: i32,
    avatar_storage_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct PlayerRankingRow {
    item_id: i32,
    slug: String,
    label: String,
    rank_no: i32,
    player_id: i64,
    player_name: String,
    team_id: i64,
    team_name: String,
    score_value: String,
    penalty_value: Option<String>,
    avatar_storage_url: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TeamInsightRow {
    team_id: i64,
    team_name: String,
    rank_no: i32,
    avatar_storage_url: Option<String>,
    goals_for_total: i32,
    goals_against_total: i32,
    goals_for_by_opponent: Json<Vec<OpponentContribution>>,
    goals_for_by_player: Json<Vec<PlayerContribution>>,
    assists_for_by_player: Json<Vec<AssistContribution>>,
    goals_against_by_opponent: Json<Vec<OpponentContribution>>,
    round_number: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::{
        OverviewPlayerRow, OverviewStandingRow, PostgresInsightQueryRepository, RoundProgressRow,
        StandingsTableRow,
    };
    use crate::insight::domain::{rankings::TeamRankingCategory, round_reference::RoundReference};
    use chrono::{NaiveDateTime, TimeZone, Utc};

    fn scorer(rank_no: i32, player_id: i64, score_value: &str) -> OverviewPlayerRow {
        OverviewPlayerRow {
            rank_no,
            player_id,
            player_name: format!("球员{player_id}"),
            team_name: "球队".to_string(),
            score_value: score_value.to_string(),
            avatar_storage_url: None,
        }
    }

    fn standing(rank_no: i32, team_id: i64, points: i32) -> OverviewStandingRow {
        OverviewStandingRow {
            rank_no,
            team_id,
            team_name: format!("球队{team_id}"),
            points,
            avatar_storage_url: Some(format!("team-{team_id}.png")),
        }
    }

    fn standings_table_row(
        rank_no: i32,
        team_id: i64,
        wins: i32,
        draws: i32,
        losses: i32,
        goals_for: i32,
        goals_against: i32,
        points: i32,
    ) -> StandingsTableRow {
        StandingsTableRow {
            rank_no,
            team_id,
            team_name: format!("球队{team_id}"),
            played: wins + draws + losses,
            wins,
            draws,
            losses,
            goals_for,
            goals_against,
            goal_difference: goals_for - goals_against,
            points,
            avatar_storage_url: Some(format!("team-{team_id}.png")),
        }
    }

    fn round_progress_row(
        round_number: i32,
        total_matches: i64,
        completed_matches: i64,
        finalized_at: Option<&str>,
    ) -> RoundProgressRow {
        RoundProgressRow {
            round_number,
            total_matches,
            completed_matches,
            finalized_at: finalized_at.map(|item| item.parse().unwrap()),
        }
    }

    #[test]
    fn keep_scors_with_boundary_ties_includes_players_tied_at_fifth() {
        let rows = vec![
            scorer(1, 1, "7"),
            scorer(2, 2, "6"),
            scorer(3, 3, "5"),
            scorer(4, 4, "4"),
            scorer(5, 5, "4"),
            scorer(6, 6, "4"),
            scorer(7, 7, "3"),
        ];

        let kept = PostgresInsightQueryRepository::keep_scorers_with_boundary_ties(rows, 5);

        assert_eq!(kept.len(), 6);
        assert_eq!(kept.last().map(|row| row.player_id), Some(6));
    }

    #[test]
    fn keep_scors_with_boundary_ties_falls_back_to_limit_when_score_is_not_numeric() {
        let rows = vec![
            scorer(1, 1, "7"),
            scorer(2, 2, "6"),
            scorer(3, 3, "5"),
            scorer(4, 4, "4"),
            scorer(5, 5, "N/A"),
            scorer(6, 6, "N/A"),
        ];

        let kept = PostgresInsightQueryRepository::keep_scorers_with_boundary_ties(rows, 5);

        assert_eq!(kept.len(), 5);
        assert_eq!(kept.last().map(|row| row.player_id), Some(5));
    }

    #[test]
    fn prepend_standings_category_adds_team_rankings_first() {
        let standings = vec![standing(1, 10, 10), standing(2, 11, 8)];
        let categories = vec![TeamRankingCategory {
            slug: "goals".to_string(),
            label: "进球".to_string(),
            item_id: 1,
            entries: Vec::new(),
        }];

        let merged =
            PostgresInsightQueryRepository::prepend_standings_category(categories, standings);

        assert_eq!(
            merged.first().map(|item| item.slug.as_str()),
            Some("standings")
        );
        assert_eq!(
            merged.first().map(|item| item.label.as_str()),
            Some("球队排名")
        );
        assert_eq!(merged.first().map(|item| item.entries.len()), Some(2));
        assert_eq!(merged.get(1).map(|item| item.slug.as_str()), Some("goals"));
    }

    #[test]
    fn build_standings_tables_splits_no_penalty_and_actual_rankings() {
        let rows = vec![
            standings_table_row(1, 11, 2, 0, 2, 5, 3, 6),
            standings_table_row(2, 12, 1, 2, 1, 4, 3, 5),
            standings_table_row(3, 10, 2, 1, 1, 8, 4, 2),
        ];

        let tables = PostgresInsightQueryRepository::build_standings_tables(&rows);

        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].slug, "standings_without_penalty");
        assert_eq!(tables[1].slug, "standings_with_penalty");

        assert_eq!(tables[0].entries[0].team_id, 10);
        assert_eq!(tables[0].entries[0].points_without_penalty, 7);
        assert_eq!(tables[0].entries[0].points_adjustment, -5);

        assert_eq!(tables[1].entries[0].team_id, 11);
        assert_eq!(tables[1].entries[1].team_id, 12);
        assert_eq!(tables[1].entries[2].team_id, 10);
    }

    #[test]
    fn summarize_rounds_marks_first_incomplete_round_as_current() {
        let rows = vec![
            round_progress_row(1, 8, 8, Some("2026-03-01T12:00:00Z")),
            round_progress_row(2, 8, 8, Some("2026-03-08T12:00:00Z")),
            round_progress_row(3, 8, 3, None),
            round_progress_row(4, 8, 0, None),
        ];

        let rounds = PostgresInsightQueryRepository::summarize_rounds(2026, rows);

        assert_eq!(rounds.len(), 4);
        assert_eq!(rounds[0].status, "completed");
        assert_eq!(rounds[1].status, "completed");
        assert_eq!(rounds[2].status, "current");
        assert_eq!(rounds[3].status, "upcoming");
        assert_eq!(rounds[2].completed_matches, 3);
        assert_eq!(rounds[3].total_matches, 8);
    }

    #[test]
    fn resolve_live_match_round_number_prefers_current_round() {
        let rounds = vec![
            RoundReference {
                season: 2026,
                round_number: 6,
                finalized_at: Some("2026-04-18T14:00:00Z".to_string()),
                status: "completed".to_string(),
                total_matches: 8,
                completed_matches: 8,
            },
            RoundReference {
                season: 2026,
                round_number: 7,
                finalized_at: None,
                status: "current".to_string(),
                total_matches: 8,
                completed_matches: 2,
            },
            RoundReference {
                season: 2026,
                round_number: 8,
                finalized_at: None,
                status: "upcoming".to_string(),
                total_matches: 8,
                completed_matches: 0,
            },
        ];

        assert_eq!(
            PostgresInsightQueryRepository::resolve_live_match_round_number(&rounds),
            Some((7, false))
        );
    }

    #[test]
    fn resolve_live_match_round_number_falls_back_to_latest_completed_round() {
        let rounds = vec![
            RoundReference {
                season: 2026,
                round_number: 29,
                finalized_at: Some("2026-10-24T10:00:00Z".to_string()),
                status: "completed".to_string(),
                total_matches: 8,
                completed_matches: 8,
            },
            RoundReference {
                season: 2026,
                round_number: 30,
                finalized_at: Some("2026-11-08T10:00:00Z".to_string()),
                status: "completed".to_string(),
                total_matches: 8,
                completed_matches: 8,
            },
        ];

        assert_eq!(
            PostgresInsightQueryRepository::resolve_live_match_round_number(&rounds),
            Some((30, true))
        );
    }

    #[test]
    fn map_match_status_maps_known_status_codes() {
        assert_eq!(
            PostgresInsightQueryRepository::map_match_status("1"),
            "scheduled"
        );
        assert_eq!(
            PostgresInsightQueryRepository::map_match_status("2"),
            "live"
        );
        assert_eq!(
            PostgresInsightQueryRepository::map_match_status("3"),
            "finished"
        );
        assert_eq!(
            PostgresInsightQueryRepository::map_match_status("99"),
            "live"
        );
    }

    #[test]
    fn china_local_naive_to_utc_converts_beijing_time_to_utc() {
        let value = NaiveDateTime::parse_from_str("2026-04-17 19:35:00", "%Y-%m-%d %H:%M:%S")
            .expect("valid naive datetime");

        let converted = PostgresInsightQueryRepository::china_local_naive_to_utc(value)
            .expect("converted datetime");

        assert_eq!(
            converted,
            Utc.with_ymd_and_hms(2026, 4, 17, 11, 35, 0).unwrap()
        );
    }
}

#[derive(sqlx::FromRow)]
struct MatchCardRow {
    match_id: i64,
    round_number: i32,
    match_date: String,
    match_time: String,
    status: String,
    home_team_id: i64,
    home_team_name: String,
    home_score: String,
    away_team_id: i64,
    away_team_name: String,
    away_score: String,
    home_team_avatar: Option<String>,
    away_team_avatar: Option<String>,
    leisu_match_id: Option<i64>,
    home_corners: Option<i32>,
    away_corners: Option<i32>,
    corner_source: Option<String>,
    technical_stats: Option<Json<Vec<MatchTechnicalStat>>>,
}

#[derive(sqlx::FromRow)]
struct RoundProgressRow {
    round_number: i32,
    total_matches: i64,
    completed_matches: i64,
    finalized_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct StandingRankRow {
    team_id: i64,
    team_name: String,
    rank_no: i32,
}

#[derive(sqlx::FromRow)]
struct PlayerRankRow {
    player_id: i64,
    player_name: String,
    rank_no: i32,
}

fn select_best_riser(
    current: Vec<(i64, String, i32)>,
    previous: Vec<(i64, String, i32)>,
) -> Option<RankingMovement> {
    use std::collections::HashMap;

    let previous_map: HashMap<i64, (String, i32)> = previous
        .into_iter()
        .map(|(id, name, rank)| (id, (name, rank)))
        .collect();

    current
        .into_iter()
        .filter_map(|(id, name, current_rank)| {
            let (_, previous_rank) = previous_map.get(&id)?.clone();
            let movement = RankingMovement {
                name,
                current_rank,
                previous_rank,
            };
            (movement.rise() > 0).then_some(movement)
        })
        .max_by_key(|movement| movement.rise())
}

#[async_trait]
impl InsightQueryRepository for PostgresInsightQueryRepository {
    async fn get_live_overview(&self) -> anyhow::Result<InsightOverview> {
        let latest_run = sqlx::query_as::<_, LatestRunRow>(
            r#"
            SELECT season, finished_at
              FROM f_i_scrape_runs
             WHERE status = 'completed'
             ORDER BY finished_at DESC NULLS LAST, started_at DESC
             LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(latest_run) = latest_run else {
            return Ok(Self::empty_overview(0, "live", None));
        };

        let mut overview = self
            .fetch_overview(
                latest_run.season,
                "live",
                None,
                latest_run.finished_at,
                "live",
                None,
                None,
                "live",
                None,
                None,
                None,
            )
            .await?;

        let standings_movement = self
            .fetch_standings_movement(latest_run.season, "live", None)
            .await?;
        let scorer_movement = self
            .fetch_scorer_movement(latest_run.season, "live", None)
            .await?;
        overview.insight_summary = Some(generate_insight_summary(
            &overview,
            standings_movement,
            scorer_movement,
        ));

        Ok(overview)
    }

    async fn get_round_overview(
        &self,
        season: i32,
        round_number: i32,
    ) -> anyhow::Result<InsightOverview> {
        let Some(finalized_at) = self
            .fetch_valid_round_finalized_at(season, round_number)
            .await?
        else {
            return Ok(Self::empty_overview(
                season,
                "round_final",
                Some(round_number),
            ));
        };

        let mut overview = self
            .fetch_overview(
                season,
                "round_final",
                Some(round_number),
                Some(finalized_at),
                "round_final",
                Some(round_number),
                Some(finalized_at),
                "round_final",
                Some(round_number),
                Some(finalized_at),
                Some(round_number),
            )
            .await?;

        let standings_movement = self
            .fetch_standings_movement(season, "round_final", Some(round_number))
            .await?;
        let scorer_movement = self
            .fetch_scorer_movement(season, "round_final", Some(round_number))
            .await?;
        overview.insight_summary = Some(generate_insight_summary(
            &overview,
            standings_movement,
            scorer_movement,
        ));

        Ok(overview)
    }

    async fn list_available_rounds(&self, season: i32) -> anyhow::Result<Vec<RoundReference>> {
        let rows = sqlx::query_as::<_, RoundProgressRow>(
            r#"
            SELECT
                round_number,
                COUNT(*) AS total_matches,
                COUNT(*) FILTER (WHERE status = '3') AS completed_matches,
                CASE
                    WHEN BOOL_AND(status = '3')
                        THEN MAX((((match_date::timestamp + match_time::time) + INTERVAL '120 minutes') AT TIME ZONE 'UTC'))
                    ELSE NULL
                END AS finalized_at
            FROM f_i_matches
            WHERE season = $1
              AND round_number IS NOT NULL
            GROUP BY round_number
            ORDER BY round_number ASC
            "#,
        )
        .bind(season)
        .fetch_all(&self.pool)
        .await?;

        Ok(Self::summarize_rounds(season, rows))
    }

    async fn get_live_rankings(&self) -> anyhow::Result<RankingsView> {
        let latest_run = sqlx::query_as::<_, LatestRunRow>(
            r#"
            SELECT season, finished_at
              FROM f_i_scrape_runs
             WHERE status = 'completed'
             ORDER BY finished_at DESC NULLS LAST, started_at DESC
             LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(latest_run) = latest_run else {
            return Ok(Self::empty_rankings(0, "live", None));
        };

        self.fetch_rankings(latest_run.season, "live", None, "live", None)
            .await
    }

    async fn get_live_team_insights(&self) -> anyhow::Result<TeamInsightsView> {
        let latest_run = sqlx::query_as::<_, LatestRunRow>(
            r#"
            SELECT season, finished_at
              FROM f_i_scrape_runs
             WHERE status = 'completed'
             ORDER BY finished_at DESC NULLS LAST, started_at DESC
             LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(latest_run) = latest_run else {
            return Ok(Self::empty_team_insights(0, "live", None));
        };

        self.fetch_live_team_insights(latest_run.season).await
    }

    async fn get_round_rankings(
        &self,
        season: i32,
        round_number: i32,
    ) -> anyhow::Result<RankingsView> {
        let Some(finalized_at) = self
            .fetch_valid_round_finalized_at(season, round_number)
            .await?
        else {
            return Ok(Self::empty_rankings(
                season,
                "round_final",
                Some(round_number),
            ));
        };

        self.fetch_rankings(
            season,
            "round_final",
            Some(round_number),
            "round_final",
            Some(finalized_at),
        )
        .await
    }

    async fn get_live_matches(&self) -> anyhow::Result<MatchListView> {
        let latest_run = sqlx::query_as::<_, LatestRunRow>(
            r#"
            SELECT season, finished_at
              FROM f_i_scrape_runs
             WHERE status = 'completed'
             ORDER BY finished_at DESC NULLS LAST, started_at DESC
             LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(latest_run) = latest_run else {
            return Ok(Self::empty_match_list(0, "live", None));
        };

        let rounds = self.list_available_rounds(latest_run.season).await?;
        let Some((round_number, finished_only)) = Self::resolve_live_match_round_number(&rounds)
        else {
            return Ok(Self::empty_match_list(latest_run.season, "live", None));
        };

        self.fetch_matches(latest_run.season, "live", Some(round_number), finished_only)
            .await
    }

    async fn get_round_matches(
        &self,
        season: i32,
        round_number: i32,
    ) -> anyhow::Result<MatchListView> {
        self.fetch_matches(season, "round", Some(round_number), false)
            .await
    }
}
