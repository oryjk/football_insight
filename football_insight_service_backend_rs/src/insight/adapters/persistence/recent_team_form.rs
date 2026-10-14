use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::insight::domain::overview::MatchOutcome;

/// Fetch only the requested teams' ten latest valid, completed matches.
/// Dates take precedence over round numbers so postponed games stay chronological.
pub(super) async fn fetch_recent_team_form(
    pool: &PgPool,
    season: i32,
    team_ids: &[i64],
    round_number: Option<i32>,
    finalized_at: Option<DateTime<Utc>>,
) -> anyhow::Result<HashMap<i64, Vec<MatchOutcome>>> {
    if team_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, TeamFormRow>(
        r#"
        SELECT teams.team_id, recent.home_team_id, recent.home_score, recent.away_score
          FROM UNNEST($2::bigint[]) AS teams(team_id)
          CROSS JOIN LATERAL (
              SELECT m.match_id, m.match_date, m.match_time,
                     m.home_team_id, m.home_score, m.away_score
                FROM f_i_matches m
               WHERE m.season = $1
                 AND m.source_active = TRUE
                 AND m.status = '3'
                 AND (m.home_team_id = teams.team_id OR m.away_team_id = teams.team_id)
                 AND BTRIM(m.home_score) ~ '^[0-9]{1,9}$'
                 AND BTRIM(m.away_score) ~ '^[0-9]{1,9}$'
                 AND ($3::int IS NULL OR m.round_number <= $3)
                 -- Same 120-minute completion estimate as round finalization.
                 AND ($4::timestamptz IS NULL OR
                      ((m.match_date::timestamp + m.match_time::time + INTERVAL '120 minutes')
                          AT TIME ZONE 'Asia/Shanghai') <= $4)
               ORDER BY m.match_date DESC, m.match_time DESC, m.match_id DESC
               LIMIT 10
          ) recent
         ORDER BY teams.team_id, recent.match_date, recent.match_time, recent.match_id
        "#,
    )
    .bind(season)
    .bind(team_ids)
    .bind(round_number)
    .bind(finalized_at)
    .fetch_all(pool)
    .await?;

    let mut forms: HashMap<i64, Vec<MatchOutcome>> = HashMap::new();
    for row in rows {
        let (goals_for, goals_against) = if row.team_id == row.home_team_id {
            (&row.home_score, &row.away_score)
        } else {
            (&row.away_score, &row.home_score)
        };
        if let Some(outcome) = MatchOutcome::from_scores(goals_for, goals_against) {
            forms.entry(row.team_id).or_default().push(outcome);
        }
    }
    Ok(forms)
}

#[derive(sqlx::FromRow)]
struct TeamFormRow {
    team_id: i64,
    home_team_id: i64,
    home_score: String,
    away_score: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn fixture() -> PgPool {
        let database_url = std::env::var("TEAM_FORM_TEST_DATABASE_URL")
            .expect("explicit disposable test database required");
        let url = url::Url::parse(&database_url).unwrap();
        assert!(matches!(url.host_str(), Some("localhost" | "127.0.0.1")));
        assert!(url.path().ends_with("_test"));
        // One connection keeps the temporary fixture private to this test.
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
                home_team_id bigint, away_team_id bigint, home_score text, away_score text
            );
            INSERT INTO f_i_matches (match_id,round_number,match_date,home_team_id,away_team_id,home_score,away_score) VALUES
                (1,8,'2026-09-01',10,20,'0','2'),
                (2,1,'2026-09-02',20,10,'1','1'),
                (3,3,'2026-09-03',10,20,'10','2'),
                (4,4,'2026-09-04',20,10,'1','2'),
                (5,5,'2026-09-05',10,20,'1','0'),
                (6,6,'2026-09-06',20,10,'1','0'),
                (7,7,'2026-09-07',10,20,'0','0'),
                (8,2,'2026-09-08',20,10,'0','1'),
                (9,9,'2026-09-09',10,20,'-','-'),
                (10,9,'2026-09-09',10,20,'0','0'),
                (11,9,'2026-09-09',10,20,'0','0'),
                (12,9,'2026-09-09',10,20,'0','0'),
                (13,9,'2026-09-09',10,20,'-1','0'),
                (14,9,'2026-09-09',10,20,'9999999999999','0'),
                (20,9,'2026-09-09',10,20,'0','0'),
                (21,10,'2026-09-10',20,10,'1','0'),
                (22,11,'2026-09-11',10,20,'2','1'),
                (23,12,'2026-09-12',20,10,'2','1');
            UPDATE f_i_matches SET status='2' WHERE match_id=10;
            UPDATE f_i_matches SET source_active=FALSE WHERE match_id=11;
            UPDATE f_i_matches SET season=2025 WHERE match_id=12;
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }

    #[tokio::test]
    #[ignore = "requires TEAM_FORM_TEST_DATABASE_URL pointing to disposable localhost _test DB"]
    async fn latest_ten_follow_match_dates_and_each_teams_home_away_perspective() {
        use MatchOutcome::{Draw, Loss, Win};
        let pool = fixture().await;
        let forms = fetch_recent_team_form(&pool, 2026, &[10, 20, 999], None, None)
            .await
            .unwrap();
        assert_eq!(
            forms[&10],
            vec![Win, Win, Win, Loss, Draw, Win, Draw, Loss, Win, Loss]
        );
        assert_eq!(
            forms[&20],
            vec![Loss, Loss, Loss, Win, Draw, Loss, Draw, Win, Loss, Win]
        );
        assert!(!forms.contains_key(&999));
        pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires TEAM_FORM_TEST_DATABASE_URL pointing to disposable localhost _test DB"]
    async fn historical_form_excludes_later_rounds_and_late_postponed_matches() {
        let pool = fixture().await;
        // This earlier-round makeup started before the snapshot but was not yet finished.
        sqlx::raw_sql("INSERT INTO f_i_matches(match_id,round_number,match_date,match_time,home_team_id,away_team_id,home_score,away_score) VALUES (15,1,'2026-09-03','22:30',10,20,'0','0')")
            .execute(&pool).await.unwrap();
        let cutoff = "2026-09-03T15:00:00Z".parse().unwrap();
        let forms = fetch_recent_team_form(&pool, 2026, &[10], Some(3), Some(cutoff))
            .await
            .unwrap();
        assert_eq!(forms[&10], vec![MatchOutcome::Draw, MatchOutcome::Win]);
        pool.close().await;
    }

    #[tokio::test]
    #[ignore = "requires TEAM_FORM_TEST_DATABASE_URL pointing to disposable localhost _test DB"]
    async fn short_form_has_no_padding_and_identical_kickoffs_have_stable_order() {
        let pool = fixture().await;
        sqlx::raw_sql("INSERT INTO f_i_matches(match_id,round_number,match_date,home_team_id,away_team_id,home_score,away_score) VALUES (901,1,'2026-10-01',30,40,'1','0'),(902,2,'2026-10-01',30,40,'0','2')")
            .execute(&pool).await.unwrap();
        let forms = fetch_recent_team_form(&pool, 2026, &[30], None, None)
            .await
            .unwrap();
        assert_eq!(forms[&30], vec![MatchOutcome::Win, MatchOutcome::Loss]);
        assert!(
            fetch_recent_team_form(&pool, 2026, &[], None, None)
                .await
                .unwrap()
                .is_empty()
        );
        pool.close().await;
    }
}
