use crate::insight::domain::overview::PlayerMatchGoals;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;

pub(super) async fn fetch_recent_player_goals(
    pool: &PgPool,
    season: i32,
    player_ids: &[i64],
    round_number: Option<i32>,
    finalized_at: Option<DateTime<Utc>>,
) -> anyhow::Result<HashMap<i64, Vec<PlayerMatchGoals>>> {
    if player_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let available: bool =
        sqlx::query_scalar("SELECT to_regclass('f_i_player_match_stats') IS NOT NULL")
            .fetch_one(pool)
            .await?;
    if !available {
        return Ok(HashMap::new());
    }
    let rows = sqlx::query_as::<_, PlayerGoalRow>(
        r#"
        SELECT players.player_id,recent.*
        FROM UNNEST($2::bigint[]) AS players(player_id)
        CROSS JOIN LATERAL (
            SELECT m.match_id,m.round_number,m.match_date::text AS match_date,m.match_time,
                   m.home_team_name,m.away_team_name,s.goals,s.minutes_played
            FROM f_i_player_match_stats s
            JOIN f_i_matches m ON m.match_id=s.match_id AND m.season=s.season
            WHERE s.player_id=players.player_id AND s.season=$1
              AND m.source_active=TRUE AND m.status='3'
              AND s.minutes_played>0 AND s.goals BETWEEN 0 AND 30
              AND ($3::int IS NULL OR m.round_number <= $3)
              AND ($4::timestamptz IS NULL OR
                   ((m.match_date::timestamp + m.match_time::time + INTERVAL '120 minutes')
                      AT TIME ZONE 'Asia/Shanghai') <= $4)
            ORDER BY m.match_date DESC,m.match_time DESC,m.match_id DESC
            LIMIT 10
        ) recent
        ORDER BY players.player_id,recent.match_date,recent.match_time,recent.match_id
    "#,
    )
    .bind(season)
    .bind(player_ids)
    .bind(round_number)
    .bind(finalized_at)
    .fetch_all(pool)
    .await?;
    let mut result: HashMap<i64, Vec<PlayerMatchGoals>> = HashMap::new();
    for row in rows {
        result
            .entry(row.player_id)
            .or_default()
            .push(PlayerMatchGoals {
                match_id: row.match_id,
                round_number: row.round_number,
                match_date: row.match_date,
                match_time: row.match_time,
                home_team_name: row.home_team_name,
                away_team_name: row.away_team_name,
                goals: row.goals,
                minutes_played: row.minutes_played,
            });
    }
    Ok(result)
}

#[derive(sqlx::FromRow)]
struct PlayerGoalRow {
    player_id: i64,
    match_id: i64,
    round_number: i32,
    match_date: String,
    match_time: String,
    home_team_name: String,
    away_team_name: String,
    goals: i32,
    minutes_played: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn fixture() -> PgPool {
        let database_url =
            std::env::var("TEAM_FORM_TEST_DATABASE_URL").expect("disposable test DB required");
        let url = url::Url::parse(&database_url).unwrap();
        assert!(
            matches!(url.host_str(), Some("127.0.0.1" | "localhost"))
                && url.path().ends_with("_test")
        );
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .unwrap();
        sqlx::raw_sql(
            r#"
            CREATE TEMP TABLE f_i_matches (
                match_id bigint PRIMARY KEY, season int DEFAULT 2026,
                source_active boolean DEFAULT TRUE, status text DEFAULT '3',
                round_number int, match_date date, match_time text DEFAULT '19:00',
                home_team_name text DEFAULT '主队',away_team_name text DEFAULT '客队'
            );
            CREATE TEMP TABLE f_i_player_match_stats (
                match_id bigint, player_id bigint, season int DEFAULT 2026,
                goals int DEFAULT 0, minutes_played int DEFAULT 90
            );
            INSERT INTO f_i_matches(match_id,round_number,match_date)
                SELECT n,n,'2026-09-01'::date + n FROM generate_series(1,15) n;
            INSERT INTO f_i_player_match_stats(match_id,player_id,goals)
                SELECT n,10,CASE WHEN n=14 THEN 3 ELSE 0 END FROM generate_series(1,15) n;
            UPDATE f_i_matches SET status='2' WHERE match_id=15;
            UPDATE f_i_matches SET source_active=FALSE WHERE match_id=13;
            UPDATE f_i_matches SET season=2025 WHERE match_id=12;
            INSERT INTO f_i_player_match_stats(match_id,player_id,goals) VALUES (2,20,0),(14,20,4);
        "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    #[ignore = "requires disposable TEAM_FORM_TEST_DATABASE_URL"]
    async fn latest_ten_preserve_zero_hat_trick_and_exclude_invalid_matches() {
        let pool = fixture().await;
        let result = fetch_recent_player_goals(&pool, 2026, &[10, 20, 99], None, None)
            .await
            .unwrap();
        assert_eq!(
            result[&10].iter().map(|r| r.match_id).collect::<Vec<_>>(),
            vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 14]
        );
        assert_eq!(result[&10][0].goals, 0);
        assert_eq!(result[&10][9].goals, 3);
        assert_eq!(
            result[&20].iter().map(|r| r.goals).collect::<Vec<_>>(),
            vec![0, 4]
        );
        assert!(!result.contains_key(&99));
        pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires disposable TEAM_FORM_TEST_DATABASE_URL"]
    async fn historical_cutoff_and_missing_table_do_not_invent_zeroes() {
        let pool = fixture().await;
        let cutoff = "2026-09-04T12:00:00Z".parse().unwrap();
        let result = fetch_recent_player_goals(&pool, 2026, &[10], Some(10), Some(cutoff))
            .await
            .unwrap();
        assert_eq!(
            result[&10].iter().map(|r| r.match_id).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert!(
            fetch_recent_player_goals(&pool, 2026, &[], None, None)
                .await
                .unwrap()
                .is_empty()
        );
        sqlx::raw_sql("DROP TABLE f_i_player_match_stats")
            .execute(&pool)
            .await
            .unwrap();
        assert!(
            fetch_recent_player_goals(&pool, 2026, &[10], None, None)
                .await
                .unwrap()
                .is_empty()
        );
        pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires disposable TEAM_FORM_TEST_DATABASE_URL"]
    async fn postponed_games_use_actual_dates_and_zero_minutes_are_not_appearances() {
        let pool = fixture().await;
        sqlx::raw_sql(
            r#"
            INSERT INTO f_i_matches(match_id,round_number,match_date) VALUES
                (98,1,'2026-10-01'),(99,2,'2026-10-01'),(100,3,'2026-10-02');
            INSERT INTO f_i_player_match_stats(match_id,player_id,goals,minutes_played) VALUES
                (98,30,0,10),(99,30,3,90),(100,30,0,0);
        "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        let result = fetch_recent_player_goals(&pool, 2026, &[30], None, None)
            .await
            .unwrap();
        assert_eq!(
            result[&30].iter().map(|r| r.match_id).collect::<Vec<_>>(),
            vec![98, 99]
        );
        pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires disposable TEAM_FORM_TEST_DATABASE_URL"]
    async fn every_requested_scorer_is_returned_beyond_the_home_top_five() {
        let pool = fixture().await;
        sqlx::raw_sql("INSERT INTO f_i_player_match_stats(match_id,player_id,goals) SELECT 14,n,3 FROM generate_series(30,49) n")
            .execute(&pool).await.unwrap();
        let ids: Vec<i64> = (30..=50).collect();
        let result = fetch_recent_player_goals(&pool, 2026, &ids, None, None)
            .await
            .unwrap();
        assert_eq!(result.len(), 20);
        assert_eq!(result[&49][0].goals, 3);
        assert!(!result.contains_key(&50));
        pool.close().await;
    }
}
