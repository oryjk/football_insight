use crate::match_outlook::{domain::*, ports::OutlookRepository};
use async_trait::async_trait;
use sqlx::PgPool;
pub struct PostgresOutlookRepository {
    pool: PgPool,
}
impl PostgresOutlookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
fn missing_table(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .is_some_and(|e| e.code().as_deref() == Some("42P01"))
}
fn unavailable(_e: sqlx::Error) -> ReadError {
    tracing::warn!("match outlook read unavailable");
    ReadError::Unavailable
}
pub const OUTLOOK_SQL: &str = r#"SELECT o.match_id,o.home_win,o.draw,o.away_win,o.tau,o.model_version,
 o.generated_at,o.kickoff_at,o.kickoff_at<=clock_timestamp() AS frozen,o.outcome,
 COALESCE((o.params->>'schedule_changed')::boolean,false) AS schedule_changed
 FROM f_i_match_outlooks o JOIN f_i_matches m USING(match_id) WHERE o.match_id=$1 AND m.source_active"#;
pub const RECORD_SQL: &str = r#"WITH scored AS (
 SELECT o.*,CASE WHEN home_win>=draw AND home_win>=away_win THEN 0
                   WHEN draw>=away_win THEN 1 ELSE 2 END AS predicted
 FROM f_i_match_outlooks o JOIN f_i_matches m USING(match_id)
 WHERE m.season=$1 AND m.source_active AND o.scored_at IS NOT NULL
) SELECT COUNT(*) AS matches,COUNT(*) FILTER(WHERE predicted=outcome) AS hits,
 AVG(CASE WHEN predicted=outcome THEN 1.0 ELSE 0.0 END)::float8 AS accuracy,
 AVG(rps_model),AVG(rps_baseline),AVG(rps_baseline)-AVG(rps_model) FROM scored"#;
#[async_trait]
impl OutlookRepository for PostgresOutlookRepository {
    async fn find(&self, id: i64) -> Result<Option<Outlook>, ReadError> {
        type Row = (
            i64,
            f64,
            f64,
            f64,
            Option<f64>,
            String,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
            bool,
            Option<i16>,
            bool,
        );
        match sqlx::query_as::<_, Row>(OUTLOOK_SQL)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
        {
            Ok(row) => Ok(row.map(
                |(
                    match_id,
                    home_win,
                    draw,
                    away_win,
                    tau,
                    model_version,
                    generated_at,
                    kickoff_at,
                    frozen,
                    outcome,
                    schedule_changed,
                )| Outlook {
                    match_id,
                    home_win,
                    draw,
                    away_win,
                    tau,
                    model_version,
                    generated_at,
                    kickoff_at,
                    frozen,
                    outcome,
                    schedule_changed,
                },
            )),
            Err(e) if missing_table(&e) => {
                tracing::warn!("match outlook migration not ready");
                Ok(None)
            }
            Err(e) => Err(unavailable(e)),
        }
    }
    async fn record(&self, season: i32) -> Result<Record, ReadError> {
        type Row = (i64, i64, Option<f64>, Option<f64>, Option<f64>, Option<f64>);
        match sqlx::query_as::<_, Row>(RECORD_SQL)
            .bind(season)
            .fetch_one(&self.pool)
            .await
        {
            Ok((matches, hits, accuracy, rps_model, rps_baseline, rps_improvement)) => Ok(Record {
                season,
                matches,
                hits,
                accuracy,
                rps_model,
                rps_baseline,
                rps_improvement,
            }),
            Err(e) if missing_table(&e) => {
                tracing::warn!("match outlook migration not ready");
                Ok(Record {
                    season,
                    matches: 0,
                    hits: 0,
                    accuracy: None,
                    rps_model: None,
                    rps_baseline: None,
                    rps_improvement: None,
                })
            }
            Err(e) => Err(unavailable(e)),
        }
    }
}
