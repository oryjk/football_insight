use super::*;

fn team(id: i64, name: &str, adjustment: i32) -> StandingsTeam {
    StandingsTeam {
        team_id: id,
        team_name: name.into(),
        avatar_storage_url: None,
        points_adjustment: adjustment,
    }
}

fn played(id: i64, date: &str, home: i64, away: i64, hg: i32, ag: i32) -> FinishedMatch {
    FinishedMatch {
        match_id: id,
        match_date: date.into(),
        match_time: "19:35".into(),
        home_team_id: home,
        away_team_id: away,
        home_goals: hg,
        away_goals: ag,
    }
}

fn table<'a>(tables: &'a [StandingsTable], slug: &str) -> &'a [StandingsTableEntry] {
    &tables.iter().find(|t| t.slug == slug).unwrap().entries
}

fn order(entries: &[StandingsTableEntry]) -> Vec<&str> {
    entries.iter().map(|e| e.team_name.as_str()).collect()
}

#[test]
fn records_points_goals_and_chronological_form() {
    let teams = [team(1, "A", 0), team(2, "B", 0), team(3, "C", 0)];
    let matches = [
        played(2, "2026-03-08", 2, 1, 1, 1),
        played(1, "2026-03-01", 1, 3, 2, 0),
        played(3, "2026-03-15", 3, 1, 3, 1),
    ];
    let tables = compute_standings_tables(&teams, &matches, "");
    let a = &table(&tables, "standings_with_penalty")
        .iter()
        .find(|e| e.team_name == "A")
        .unwrap()
        .clone();
    assert_eq!((a.played, a.wins, a.draws, a.losses), (3, 1, 1, 1));
    assert_eq!(
        (a.goals_for, a.goals_against, a.goal_difference, a.points),
        (4, 4, 0, 4)
    );
    assert_eq!(
        a.recent_form,
        vec![MatchOutcome::Win, MatchOutcome::Draw, MatchOutcome::Loss]
    );
}

#[test]
fn teams_without_matches_still_appear_with_zero_record() {
    let tables = compute_standings_tables(&[team(1, "A", 0), team(2, "B", -3)], &[], "");
    let entries = table(&tables, "standings_with_penalty");
    assert_eq!(order(entries), ["A", "B"]);
    assert_eq!((entries[1].played, entries[1].points), (0, -3));
    assert!(compute_standings_tables(&[], &[], "").is_empty());
}

#[test]
fn penalty_table_uses_adjusted_points_and_other_table_raw_points() {
    // A wins twice (6 raw, -5 → 1); B draws once (1 raw); C loses twice.
    let teams = [team(1, "A", -5), team(2, "B", 0), team(3, "C", 0)];
    let matches = [
        played(1, "2026-03-01", 1, 3, 1, 0),
        played(2, "2026-03-08", 3, 1, 0, 2),
        played(3, "2026-03-15", 2, 2_000, 0, 0),
    ];
    let tables = compute_standings_tables(&teams, &matches, " 尾注");
    assert_eq!(
        order(table(&tables, "standings_without_penalty")),
        ["A", "B", "C"]
    );
    let with_penalty = table(&tables, "standings_with_penalty");
    // A and B both on 1 point with no direct meeting, so overall goal difference decides.
    assert_eq!(order(with_penalty), ["A", "B", "C"]);
    assert_eq!(with_penalty[0].points_adjustment, -5);
    assert_eq!(with_penalty[0].points_without_penalty, 6);
    assert!(tables[1].note.ends_with(" 尾注"));
    assert_eq!(
        with_penalty.iter().map(|e| e.rank_no).collect::<Vec<_>>(),
        [1, 2, 3]
    );
}

#[test]
fn head_to_head_decides_once_both_meetings_are_played() {
    // A and B finish on 4 points; B has the better goal difference (+6 vs 0)
    // but A took 4 points from their two meetings, so A ranks first.
    let teams = [team(1, "A", 0), team(2, "B", 0), team(3, "C", 0)];
    let matches = [
        played(1, "2026-03-01", 1, 2, 1, 0),
        played(2, "2026-03-08", 2, 1, 0, 0),
        played(3, "2026-03-15", 2, 3, 7, 0),
        played(4, "2026-03-22", 3, 1, 1, 0),
    ];
    let tables = compute_standings_tables(&teams, &matches, "");
    let entries = table(&tables, "standings_with_penalty");
    assert_eq!((entries[0].points, entries[1].points), (4, 4));
    assert_eq!(order(entries), ["A", "B", "C"]);
}

#[test]
fn incomplete_head_to_head_is_ignored_in_favour_of_goal_difference() {
    // A, B and C all on 3 points, but no pair has met twice: Sina orders by overall
    // goal difference until both meetings are played (round 4 of 2026: 上海海港
    // ahead of 河南队 on goal difference despite losing their only meeting).
    let teams = [team(1, "A", 0), team(2, "B", 0), team(3, "C", 0)];
    let matches = [
        played(1, "2026-03-01", 1, 2, 1, 0),
        played(3, "2026-03-15", 2, 3, 7, 0),
        played(4, "2026-03-22", 3, 1, 1, 0),
    ];
    let tables = compute_standings_tables(&teams, &matches, "");
    assert_eq!(order(table(&tables, "standings_with_penalty")), ["B", "A", "C"]);
}

#[test]
fn three_way_tie_uses_the_mini_league_when_all_pairs_met_twice() {
    // Each pair splits its two meetings 1-0 / 0-1 except B, who beats C 5-0
    // at home: all on 9 points, mini-league goal difference B +4, A 0, C -4.
    let teams = [team(1, "A", 0), team(2, "B", 0), team(3, "C", 0)];
    let matches = [
        played(1, "2026-03-01", 1, 2, 1, 0),
        played(2, "2026-03-08", 2, 1, 1, 0),
        played(3, "2026-03-15", 2, 3, 5, 0),
        played(4, "2026-03-22", 3, 2, 1, 0),
        played(5, "2026-03-29", 3, 1, 1, 0),
        played(6, "2026-04-05", 1, 3, 1, 0),
    ];
    let tables = compute_standings_tables(&teams, &matches, "");
    let entries = table(&tables, "standings_with_penalty");
    assert!(entries.iter().all(|e| e.points == 6));
    assert_eq!(order(entries), ["B", "A", "C"]);
}

#[test]
fn falls_back_to_overall_goal_difference_then_goals_when_head_to_head_is_level() {
    let teams = [
        team(1, "A", 0),
        team(2, "B", 0),
        team(3, "C", 0),
        team(4, "D", 0),
    ];
    let matches = [
        played(1, "2026-03-01", 1, 2, 1, 1), // A-B level
        played(2, "2026-03-08", 1, 3, 1, 0), // A +1
        played(3, "2026-03-08", 2, 4, 3, 0), // B +3
    ];
    let tables = compute_standings_tables(&teams, &matches, "");
    assert_eq!(
        order(table(&tables, "standings_with_penalty"))[..2],
        ["B", "A"]
    );
}

#[test]
fn snapshot_agreement_compares_matches_played_per_team() {
    let teams = [team(1, "A", 0), team(2, "B", 0)];
    let computed = compute_standings_tables(&teams, &[played(1, "2026-03-01", 1, 2, 1, 0)], "");
    let mut stale = computed.clone();
    stale[1].entries[0].played = 0;
    assert!(snapshot_agrees_with(&computed, &computed));
    assert!(!snapshot_agrees_with(&stale, &computed));
    assert!(!snapshot_agrees_with(&[], &computed));

    let category = standings_category(&computed).unwrap();
    assert_eq!((category.slug.as_str(), category.item_id), ("standings", 0));
    assert_eq!(category.entries[0].team_name, "A");
    assert_eq!(category.entries[0].score_value, "3");
}
