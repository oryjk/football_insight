//! Standings recomputed from match results.
//!
//! Sina updates its standings table minutes to hours after it marks a match
//! finished, so `round_final` standings snapshots are frequently stale. Match
//! results are reliable, so "standings after round N" is derived from them.

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use super::{
    overview::MatchOutcome,
    rankings::{StandingsTable, StandingsTableEntry, TeamRankingCategory, TeamRankingEntry},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedMatch {
    pub match_id: i64,
    /// `YYYY-MM-DD`, used for chronological ordering of recent form.
    pub match_date: String,
    /// `HH:MM`.
    pub match_time: String,
    pub home_team_id: i64,
    pub away_team_id: i64,
    pub home_goals: i32,
    pub away_goals: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandingsTeam {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_storage_url: Option<String>,
    /// Points deducted (negative) or awarded, taken from the official table.
    pub points_adjustment: i32,
}

#[derive(Debug, Clone, Default)]
struct Record {
    played: i32,
    wins: i32,
    draws: i32,
    losses: i32,
    goals_for: i32,
    goals_against: i32,
}

impl Record {
    fn add(&mut self, scored: i32, conceded: i32) {
        self.played += 1;
        self.goals_for += scored;
        self.goals_against += conceded;
        match scored.cmp(&conceded) {
            Ordering::Greater => self.wins += 1,
            Ordering::Equal => self.draws += 1,
            Ordering::Less => self.losses += 1,
        }
    }

    fn raw_points(&self) -> i32 {
        self.wins * 3 + self.draws
    }

    fn goal_difference(&self) -> i32 {
        self.goals_for - self.goals_against
    }
}

/// Builds the "without penalty" and "with penalty" tables, ranked by points;
/// ties are broken by head-to-head (only once the tied teams have played each
/// other home and away), then overall goal difference and goals scored.
pub fn compute_standings_tables(
    teams: &[StandingsTeam],
    matches: &[FinishedMatch],
    note_suffix: &str,
) -> Vec<StandingsTable> {
    if teams.is_empty() {
        return Vec::new();
    }
    let mut ordered: Vec<&FinishedMatch> = matches.iter().collect();
    ordered.sort_by(|a, b| {
        (a.match_date.as_str(), a.match_time.as_str(), a.match_id).cmp(&(
            b.match_date.as_str(),
            b.match_time.as_str(),
            b.match_id,
        ))
    });

    let mut records: HashMap<i64, Record> = HashMap::new();
    let mut forms: HashMap<i64, Vec<MatchOutcome>> = HashMap::new();
    for m in &ordered {
        records
            .entry(m.home_team_id)
            .or_default()
            .add(m.home_goals, m.away_goals);
        records
            .entry(m.away_team_id)
            .or_default()
            .add(m.away_goals, m.home_goals);
        forms
            .entry(m.home_team_id)
            .or_default()
            .push(outcome(m.home_goals, m.away_goals));
        forms
            .entry(m.away_team_id)
            .or_default()
            .push(outcome(m.away_goals, m.home_goals));
    }

    let entry = |team: &StandingsTeam| {
        let record = records.get(&team.team_id).cloned().unwrap_or_default();
        StandingsTableEntry {
            rank_no: 0,
            team_id: team.team_id,
            team_name: team.team_name.clone(),
            played: record.played,
            wins: record.wins,
            draws: record.draws,
            losses: record.losses,
            goals_for: record.goals_for,
            goals_against: record.goals_against,
            goal_difference: record.goal_difference(),
            points: record.raw_points() + team.points_adjustment,
            points_without_penalty: record.raw_points(),
            points_adjustment: team.points_adjustment,
            avatar_storage_url: team.avatar_storage_url.clone(),
            recent_form: forms.get(&team.team_id).cloned().unwrap_or_default(),
        }
    };

    let without_penalty = rank(teams.iter().map(entry).collect(), &ordered, |e| {
        e.points_without_penalty
    });
    let with_penalty = rank(teams.iter().map(entry).collect(), &ordered, |e| e.points);

    vec![
        StandingsTable {
            slug: "standings_without_penalty".to_string(),
            label: "无罚分版积分榜".to_string(),
            note: format!("按胜平负推导的理论积分排序，不计入罚分调整。{note_suffix}"),
            entries: without_penalty,
        },
        StandingsTable {
            slug: "standings_with_penalty".to_string(),
            label: "含罚分版积分榜".to_string(),
            note: format!("按实际积分排序，已包含罚分影响。{note_suffix}"),
            entries: with_penalty,
        },
    ]
}

/// True when a standings snapshot counts the same number of matches per team as
/// the recomputed table, i.e. Sina's table had caught up when it was taken. Only
/// then are the snapshot's other leaderboards trusted for that round.
pub fn snapshot_agrees_with(snapshot: &[StandingsTable], computed: &[StandingsTable]) -> bool {
    let played = |tables: &[StandingsTable]| -> Option<HashMap<i64, i32>> {
        let table = tables.iter().find(|t| t.slug == "standings_with_penalty")?;
        Some(
            table
                .entries
                .iter()
                .map(|e| (e.team_id, e.played))
                .collect(),
        )
    };
    match (played(snapshot), played(computed)) {
        (Some(snapshot), Some(computed)) => !snapshot.is_empty() && snapshot == computed,
        _ => false,
    }
}

/// The "球队排名" team category that leads `team_categories`.
pub fn standings_category(computed: &[StandingsTable]) -> Option<TeamRankingCategory> {
    let table = computed
        .iter()
        .find(|t| t.slug == "standings_with_penalty")?;
    Some(TeamRankingCategory {
        slug: "standings".to_string(),
        label: "球队排名".to_string(),
        item_id: 0,
        entries: table
            .entries
            .iter()
            .map(|e| TeamRankingEntry {
                rank_no: e.rank_no,
                team_id: e.team_id,
                team_name: e.team_name.clone(),
                score_value: e.points.to_string(),
                avatar_storage_url: e.avatar_storage_url.clone(),
            })
            .collect(),
    })
}

fn outcome(scored: i32, conceded: i32) -> MatchOutcome {
    match scored.cmp(&conceded) {
        Ordering::Greater => MatchOutcome::Win,
        Ordering::Equal => MatchOutcome::Draw,
        Ordering::Less => MatchOutcome::Loss,
    }
}

fn rank(
    mut entries: Vec<StandingsTableEntry>,
    matches: &[&FinishedMatch],
    points: impl Fn(&StandingsTableEntry) -> i32,
) -> Vec<StandingsTableEntry> {
    entries.sort_by_key(|e| std::cmp::Reverse(points(e)));
    let mut ranked = Vec::with_capacity(entries.len());
    let mut start = 0;
    while start < entries.len() {
        let level = points(&entries[start]);
        let end = entries[start..]
            .iter()
            .position(|e| points(e) != level)
            .map_or(entries.len(), |offset| start + offset);
        let mut group = entries[start..end].to_vec();
        if group.len() > 1 {
            break_ties(&mut group, matches);
        }
        ranked.extend(group);
        start = end;
    }
    for (index, entry) in ranked.iter_mut().enumerate() {
        entry.rank_no = index as i32 + 1;
    }
    ranked
}

/// Meetings per pair in a double round-robin season.
const MEETINGS_PER_PAIR: usize = 2;

/// Head-to-head records decide ties only once every pair of tied teams has
/// played both of its meetings; until then the official table (Sina) orders
/// tied teams by overall goal difference, then goals scored.
fn break_ties(group: &mut [StandingsTableEntry], matches: &[&FinishedMatch]) {
    let ids: HashSet<i64> = group.iter().map(|e| e.team_id).collect();
    let mut head_to_head: HashMap<i64, Record> = HashMap::new();
    let mut meetings: HashMap<(i64, i64), usize> = HashMap::new();
    for m in matches
        .iter()
        .filter(|m| ids.contains(&m.home_team_id) && ids.contains(&m.away_team_id))
    {
        head_to_head
            .entry(m.home_team_id)
            .or_default()
            .add(m.home_goals, m.away_goals);
        head_to_head
            .entry(m.away_team_id)
            .or_default()
            .add(m.away_goals, m.home_goals);
        let pair = (
            m.home_team_id.min(m.away_team_id),
            m.home_team_id.max(m.away_team_id),
        );
        *meetings.entry(pair).or_default() += 1;
    }
    let pairs = group.len() * (group.len() - 1) / 2;
    let head_to_head_complete = meetings.len() == pairs
        && meetings.values().all(|count| *count >= MEETINGS_PER_PAIR);
    let key = |e: &StandingsTableEntry| {
        let h2h = head_to_head_complete
            .then(|| head_to_head.get(&e.team_id).cloned())
            .flatten()
            .unwrap_or_default();
        (
            h2h.raw_points(),
            h2h.goal_difference(),
            h2h.goals_for,
            e.goal_difference,
            e.goals_for,
        )
    };
    group.sort_by(|a, b| key(b).cmp(&key(a)).then_with(|| a.team_id.cmp(&b.team_id)));
}

#[cfg(test)]
#[path = "round_standings_tests.rs"]
mod tests;
