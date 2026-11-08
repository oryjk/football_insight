use sqlx::PgPool;

use crate::insight::domain::round_standings::{FinishedMatch, StandingsTeam};

pub(super) struct RoundStandingsInput {
    pub teams: Vec<StandingsTeam>,
    pub matches: Vec<FinishedMatch>,
}

/// Inputs for "standings after round N", or `None` when round N is not over yet.
/// A round is over once all its matches are finished, or once a later round has
/// started (a postponed match must not hold back the whole table).
///
/// The table is the one that stood when round N ended: matches of rounds 1..=N
/// that kicked off before round N+1 began. A postponed match played later is
/// left out, exactly as in the official table at that time.
pub(super) async fn fetch_round_standings_input(
    pool: &PgPool,
    season: i32,
    round_number: i32,
) -> anyhow::Result<Option<RoundStandingsInput>> {
    let (total, finished, later_started) = sqlx::query_as::<_, (i64, i64, bool)>(
        r#"
        SELECT COUNT(*) FILTER (WHERE round_number = $2),
               COUNT(*) FILTER (WHERE round_number = $2 AND status = '3'),
               COALESCE(BOOL_OR(round_number > $2 AND status IN ('2', '3')), FALSE)
          FROM f_i_matches
         WHERE season = $1
           AND source_active = TRUE
        "#,
    )
    .bind(season)
    .bind(round_number)
    .fetch_one(pool)
    .await?;
    if total == 0 || (finished < total && !later_started) {
        return Ok(None);
    }

    let matches = sqlx::query_as::<_, MatchRow>(
        r#"
        SELECT match_id, match_date::text AS match_date, match_time,
               home_team_id, away_team_id,
               BTRIM(home_score)::int AS home_goals, BTRIM(away_score)::int AS away_goals
          FROM f_i_matches
         WHERE season = $1
           AND source_active = TRUE
           AND status = '3'
           AND round_number <= $2
           AND BTRIM(home_score) ~ '^[0-9]{1,4}$'
           AND BTRIM(away_score) ~ '^[0-9]{1,4}$'
           AND (match_date::timestamp + match_time::time) < COALESCE(
                   (SELECT MIN(n.match_date::timestamp + n.match_time::time)
                      FROM f_i_matches n
                     WHERE n.season = $1
                       AND n.source_active = TRUE
                       AND n.round_number > $2),
                   'infinity'::timestamp)
        "#,
    )
    .bind(season)
    .bind(round_number)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| FinishedMatch {
        match_id: row.match_id,
        match_date: row.match_date,
        match_time: row.match_time,
        home_team_id: row.home_team_id,
        away_team_id: row.away_team_id,
        home_goals: row.home_goals,
        away_goals: row.away_goals,
    })
    .collect();

    // Point deductions are only published in the official table: take them from
    // the latest live standings (deductions are applied for the whole season).
    let teams = sqlx::query_as::<_, TeamRow>(
        r#"
        WITH season_teams AS (
            SELECT home_team_id AS team_id, home_team_name AS team_name
              FROM f_i_matches WHERE season = $1 AND source_active = TRUE
            UNION
            SELECT away_team_id, away_team_name
              FROM f_i_matches WHERE season = $1 AND source_active = TRUE
        ),
        latest_live AS (
            SELECT MAX(snapshot_at) AS snapshot_at
              FROM f_i_standings
             WHERE season = $1 AND snapshot_kind = 'live'
        )
        SELECT DISTINCT ON (st.team_id)
               st.team_id,
               COALESCE(ls.team_name, st.team_name) AS team_name,
               t.avatar_storage_url,
               COALESCE(ls.points - (ls.wins * 3 + ls.draws), 0)::int AS points_adjustment
          FROM season_teams st
          LEFT JOIN f_i_teams t ON t.team_id = st.team_id
          LEFT JOIN f_i_standings ls
                 ON ls.season = $1
                AND ls.snapshot_kind = 'live'
                AND ls.team_id = st.team_id
                AND ls.snapshot_at = (SELECT snapshot_at FROM latest_live)
         WHERE st.team_id IS NOT NULL
         ORDER BY st.team_id
        "#,
    )
    .bind(season)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| StandingsTeam {
        team_id: row.team_id,
        team_name: row.team_name,
        avatar_storage_url: row.avatar_storage_url,
        points_adjustment: row.points_adjustment,
    })
    .collect();

    Ok(Some(RoundStandingsInput { teams, matches }))
}

#[derive(sqlx::FromRow)]
struct MatchRow {
    match_id: i64,
    match_date: String,
    match_time: String,
    home_team_id: i64,
    away_team_id: i64,
    home_goals: i32,
    away_goals: i32,
}

#[derive(sqlx::FromRow)]
struct TeamRow {
    team_id: i64,
    team_name: String,
    avatar_storage_url: Option<String>,
    points_adjustment: i32,
}
