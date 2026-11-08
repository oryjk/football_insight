use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;

use super::*;
use crate::insight::domain::{
    match_list::{MatchCard, MatchListView},
    overview::{InsightOverview, MatchOutcome},
    rankings::{
        PlayerRankingCategory, PlayerRankingEntry, RankingsView, StandingsTable,
        StandingsTableEntry,
    },
    round_reference::RoundReference,
    team_insight::TeamInsightsView,
};

#[derive(Default)]
pub(crate) struct FakeRepository {
    pub rankings_calls: AtomicUsize,
    pub round_calls: AtomicUsize,
}

pub(crate) fn standings_entry(rank: i32, team: &str, points: i32, adj: i32) -> StandingsTableEntry {
    StandingsTableEntry {
        rank_no: rank,
        team_id: rank as i64,
        team_name: team.into(),
        played: 30,
        wins: 15,
        draws: 7,
        losses: 8,
        goals_for: 50,
        goals_against: 30,
        goal_difference: 20,
        points,
        points_without_penalty: points - adj,
        points_adjustment: adj,
        avatar_storage_url: None,
        recent_form: vec![
            MatchOutcome::Loss,
            MatchOutcome::Win,
            MatchOutcome::Win,
            MatchOutcome::Draw,
            MatchOutcome::Loss,
            MatchOutcome::Win,
        ],
    }
}

fn player(rank: i32, name: &str, value: &str, penalty: Option<&str>) -> PlayerRankingEntry {
    PlayerRankingEntry {
        rank_no: rank,
        player_id: rank as i64,
        player_name: name.into(),
        team_id: 1,
        team_name: "成都蓉城".into(),
        score_value: value.into(),
        penalty_value: penalty.map(Into::into),
        avatar_storage_url: None,
        recent_goals: vec![],
    }
}

pub(crate) fn card(round: i32, status: &str, home: &str, away: &str) -> MatchCard {
    MatchCard {
        match_id: 1,
        round_number: round,
        match_date: "2026-11-08".into(),
        match_time: "15:30".into(),
        status: status.into(),
        home_team_id: 1,
        home_team_name: "上海海港".into(),
        home_score: home.into(),
        away_team_id: 2,
        away_team_name: "辽宁铁人".into(),
        away_score: away.into(),
        home_team_avatar: None,
        away_team_avatar: None,
        leisu_match_id: None,
        home_corners: None,
        away_corners: None,
        corner_source: None,
        technical_stats: vec![],
    }
}

#[async_trait]
impl InsightQueryRepository for FakeRepository {
    async fn get_live_overview(&self) -> anyhow::Result<InsightOverview> {
        unreachable!()
    }
    async fn get_round_overview(&self, _: i32, _: i32) -> anyhow::Result<InsightOverview> {
        unreachable!()
    }
    async fn list_available_rounds(&self, season: i32) -> anyhow::Result<Vec<RoundReference>> {
        Ok((1..=30)
            .map(|round_number| RoundReference {
                season,
                round_number,
                finalized_at: None,
                status: "completed".into(),
                total_matches: 8,
                completed_matches: 8,
            })
            .collect())
    }
    async fn get_live_rankings(&self) -> anyhow::Result<RankingsView> {
        self.rankings_calls.fetch_add(1, Ordering::SeqCst);
        Ok(RankingsView {
            view_kind: "live".into(),
            round_number: None,
            current_season: 2026,
            standings_tables: vec![
                StandingsTable {
                    slug: "standings_without_penalty".into(),
                    label: "无罚分版积分榜".into(),
                    note: String::new(),
                    entries: vec![standings_entry(1, "北京国安", 55, 0)],
                },
                StandingsTable {
                    slug: "standings_with_penalty".into(),
                    label: "含罚分版积分榜".into(),
                    note: "按当前实际积分排序".into(),
                    entries: vec![
                        standings_entry(1, "大连英博", 52, 0),
                        standings_entry(2, "北京国安", 50, -5),
                    ],
                },
            ],
            team_categories: vec![],
            player_categories: vec![
                PlayerRankingCategory {
                    slug: "goals".into(),
                    label: "进球".into(),
                    item_id: 1,
                    entries: vec![
                        player(1, "奥斯卡", "23", Some("2")),
                        player(2, "费利佩", "18", Some("")),
                        player(3, "韦世豪", "12", None),
                    ],
                },
                PlayerRankingCategory {
                    slug: "assists".into(),
                    label: "助攻".into(),
                    item_id: 2,
                    entries: vec![],
                },
            ],
        })
    }
    async fn get_live_team_insights(&self) -> anyhow::Result<TeamInsightsView> {
        unreachable!()
    }
    async fn get_round_rankings(&self, _: i32, _: i32) -> anyhow::Result<RankingsView> {
        unreachable!()
    }
    async fn get_live_matches(&self) -> anyhow::Result<MatchListView> {
        Ok(MatchListView {
            view_kind: "live".into(),
            round_number: Some(30),
            current_season: 2026,
            matches: vec![card(30, "finished", "6", "1")],
        })
    }
    async fn get_round_matches(&self, season: i32, round: i32) -> anyhow::Result<MatchListView> {
        self.round_calls.fetch_add(1, Ordering::SeqCst);
        Ok(MatchListView {
            view_kind: "round".into(),
            round_number: Some(round),
            current_season: season,
            matches: if round == 7 {
                vec![]
            } else {
                vec![
                    card(round, "finished", "2", "2"),
                    card(round, "scheduled", "", ""),
                    card(round, "live", " 1 ", "0"),
                ]
            },
        })
    }
}

fn service(repo: Arc<FakeRepository>) -> McpQueryService {
    McpQueryService::new(
        repo,
        "https://match.example/".into(),
        Duration::from_secs(60),
    )
}

#[tokio::test]
async fn standings_use_the_penalty_table_and_last_five_results() {
    let result = service(Arc::default()).standings().await.unwrap();
    assert_eq!(result.table, "含罚分版积分榜");
    assert_eq!(result.season, 2026);
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[1].team, "北京国安");
    assert_eq!(result.rows[1].points_adjustment, -5);
    assert_eq!(result.rows[0].recent_form, "胜胜平负胜");
    assert_eq!(
        result.promo.landing_url,
        "https://match.example/api/v1/mcp/promo/rankings"
    );
    assert_eq!(
        result.promo.code_image_url,
        "https://match.example/api/v1/mcp/promo/rankings/code"
    );
}

#[tokio::test]
async fn round_results_default_to_latest_round_and_point_to_home() {
    let repo = Arc::new(FakeRepository::default());
    let result = service(repo.clone()).round_results(None).await.unwrap();
    assert_eq!((result.round, result.latest_round), (30, 30));
    assert_eq!(result.matches[0].home_score.as_deref(), Some("6"));
    assert_eq!(result.matches[0].status, "已结束");
    assert!(result.promo.landing_url.ends_with("/promo/home"));
    assert_eq!(repo.round_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn round_results_hide_scores_of_unplayed_matches() {
    let result = service(Arc::default())
        .round_results(Some(12))
        .await
        .unwrap();
    assert_eq!(result.round, 12);
    assert_eq!(result.latest_round, 30);
    let [finished, scheduled, live] = result.matches.as_slice() else {
        panic!("expected three matches");
    };
    assert_eq!(finished.home_score.as_deref(), Some("2"));
    assert_eq!(
        (scheduled.home_score.clone(), scheduled.status.as_str()),
        (None, "未开赛")
    );
    assert_eq!(live.home_score.as_deref(), Some("1"));
    assert_eq!(live.status, "进行中");
}

#[tokio::test]
async fn round_results_reject_out_of_range_and_report_empty_rounds() {
    let svc = service(Arc::default());
    for round in [0, 31, -3] {
        let error = svc.round_results(Some(round)).await.unwrap_err();
        assert!(
            matches!(&error, McpQueryError::InvalidArgument(msg) if msg.contains("1–30")),
            "{error:?}"
        );
    }
    assert!(matches!(
        svc.round_results(Some(7)).await.unwrap_err(),
        McpQueryError::NotFound(_)
    ));
}

#[tokio::test]
async fn queries_are_cached_within_ttl() {
    let repo = Arc::new(FakeRepository::default());
    let svc = service(repo.clone());
    svc.standings().await.unwrap();
    svc.player_leaderboard("goals", None).await.unwrap();
    svc.round_results(Some(12)).await.unwrap();
    svc.round_results(Some(12)).await.unwrap();
    assert_eq!(repo.rankings_calls.load(Ordering::SeqCst), 1);
    assert_eq!(repo.round_calls.load(Ordering::SeqCst), 1);

    let expired = McpQueryService::new(repo.clone(), "https://x".into(), Duration::ZERO);
    expired.standings().await.unwrap();
    expired.standings().await.unwrap();
    assert_eq!(repo.rankings_calls.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn player_leaderboard_limits_rows_and_keeps_penalties() {
    let result = service(Arc::default())
        .player_leaderboard("goals", Some(2))
        .await
        .unwrap();
    assert_eq!(result.category_label, "进球榜");
    assert_eq!(result.rows.len(), 2);
    assert_eq!(result.rows[0].penalties.as_deref(), Some("2"));
    assert_eq!(result.rows[1].penalties, None);
}

#[tokio::test]
async fn player_leaderboard_validates_arguments_and_empty_categories() {
    let svc = service(Arc::default());
    for (category, limit) in [("unknown", None), ("goals", Some(0)), ("goals", Some(51))] {
        assert!(matches!(
            svc.player_leaderboard(category, limit).await.unwrap_err(),
            McpQueryError::InvalidArgument(_)
        ));
    }
    assert!(matches!(
        svc.player_leaderboard("assists", None).await.unwrap_err(),
        McpQueryError::NotFound(_)
    ));
}
