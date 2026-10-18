use super::super::{
    domain::{RecapError, RecapMatch},
    ports::{RecapRepository, Snapshot, ValidateSnapshot},
};
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
pub struct PostgresRecapRepository {
    pool: PgPool,
}
impl PostgresRecapRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
// An exact first-team name prevents B-team matches entering either selection or mileage.
pub const MATCHES_SQL: &str = r#"SELECT match_id,match_date::text AS match_date,
 CASE WHEN home_team_name='成都蓉城' THEN away_team_name ELSE home_team_name END AS opponent,
 home_team_name='成都蓉城' AS is_home FROM f_i_matches
 WHERE season=$1 AND source_active=TRUE AND status='3'
 AND (home_team_name='成都蓉城' OR away_team_name='成都蓉城')
 ORDER BY match_date,match_time,match_id"#;
pub const PREF_SQL: &str = "SELECT home_one_way_km FROM f_i_season_recap_profiles WHERE user_id=$1";
pub const SELECTED_SQL: &str = "SELECT a.match_id FROM f_i_season_recap_attendances a JOIN f_i_matches m USING(match_id) WHERE a.user_id=$1 AND m.season=$2 ORDER BY a.match_id";
pub const UPSERT_SQL: &str = "INSERT INTO f_i_season_recap_profiles(user_id,home_one_way_km) VALUES($1,$2) ON CONFLICT(user_id) DO UPDATE SET home_one_way_km=COALESCE(EXCLUDED.home_one_way_km,f_i_season_recap_profiles.home_one_way_km)";
pub const DELETE_SQL: &str = "DELETE FROM f_i_season_recap_attendances a USING f_i_matches m WHERE a.match_id=m.match_id AND a.user_id=$1 AND m.season=$2";
pub const INSERT_SQL: &str = "INSERT INTO f_i_season_recap_attendances(user_id,match_id) SELECT $1,unnest($2::bigint[]) ON CONFLICT(user_id,match_id) DO NOTHING";
#[derive(sqlx::FromRow)]
struct MatchRow {
    match_id: i64,
    match_date: String,
    opponent: String,
    is_home: bool,
}
async fn snapshot(
    tx: &mut Transaction<'_, Postgres>,
    user: Uuid,
    season: i32,
) -> Result<Snapshot, RecapError> {
    snapshot_with_lock(tx, user, season, false).await
}
async fn snapshot_with_lock(
    tx: &mut Transaction<'_, Postgres>,
    user: Uuid,
    season: i32,
    lock: bool,
) -> Result<Snapshot, RecapError> {
    let query = if lock {
        format!("{MATCHES_SQL} FOR SHARE")
    } else {
        MATCHES_SQL.to_string()
    };
    let matches = sqlx::query_as::<_, MatchRow>(&query)
        .bind(season)
        .fetch_all(&mut **tx)
        .await
        .map_err(db_error)?
        .into_iter()
        .map(|m| RecapMatch {
            match_id: m.match_id,
            match_date: m.match_date,
            opponent: m.opponent,
            is_home: m.is_home,
        })
        .collect();
    let home_one_way_km = sqlx::query_scalar::<_, Option<f64>>(PREF_SQL)
        .bind(user)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db_error)?
        .flatten();
    let selected = sqlx::query_scalar(SELECTED_SQL)
        .bind(user)
        .bind(season)
        .fetch_all(&mut **tx)
        .await
        .map_err(db_error)?;
    Ok(Snapshot {
        matches,
        selected,
        home_one_way_km,
    })
}
fn db_error(error: sqlx::Error) -> RecapError {
    let code = error
        .as_database_error()
        .and_then(|db| db.code())
        .map(|code| code.into_owned());
    tracing::warn!(sqlstate=?code,"season recap database unavailable");
    RecapError::Unavailable
}
#[async_trait]
impl RecapRepository for PostgresRecapRepository {
    async fn load(&self, user: Uuid, season: i32) -> Result<Snapshot, RecapError> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        let result = snapshot(&mut tx, user, season).await?;
        tx.commit().await.map_err(db_error)?;
        Ok(result)
    }
    async fn save(
        &self,
        user: Uuid,
        season: i32,
        selected: &[i64],
        home_km: Option<f64>,
        validate: &ValidateSnapshot<'_>,
    ) -> Result<Snapshot, RecapError> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        // Upsert serializes concurrent saves for the same user, including first save.
        sqlx::query(UPSERT_SQL)
            .bind(user)
            .bind(home_km)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        let mut current = snapshot_with_lock(&mut tx, user, season, true).await?;
        if selected
            .iter()
            .any(|id| !current.matches.iter().any(|m| m.match_id == *id))
        {
            return Err(RecapError::InvalidMatch);
        }
        current.selected = selected.to_vec();
        validate(&current)?;
        sqlx::query(DELETE_SQL)
            .bind(user)
            .bind(season)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        sqlx::query(INSERT_SQL)
            .bind(user)
            .bind(selected)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        current.selected = selected
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    async fn save(
        repo: &PostgresRecapRepository,
        user: Uuid,
        season: i32,
        selected: &[i64],
        distance: Option<f64>,
    ) -> Result<Snapshot, RecapError> {
        repo.save(user, season, selected, distance, &|s| {
            crate::season_recap::application::calculate(&s.matches, &s.selected, s.home_one_way_km)
                .map(|_| ())
        })
        .await
    }
    #[tokio::test]
    #[ignore = "requires dedicated disposable season_recap_test database, never production"]
    async fn isolated_migration_and_repository() {
        let url = std::env::var("SEASON_RECAP_TEST_DATABASE_URL")
            .expect("explicit isolated database URL required");
        let parsed = url::Url::parse(&url).unwrap();
        assert_eq!(parsed.host_str(), Some("127.0.0.1"));
        assert_eq!(parsed.path(), "/season_recap_test");
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::raw_sql("CREATE TABLE f_i_users(id UUID PRIMARY KEY); CREATE TABLE f_i_matches(match_id BIGINT PRIMARY KEY,season INT,match_date DATE,match_time TEXT,home_team_name TEXT,away_team_name TEXT,status TEXT,source_active BOOL);").execute(&pool).await.unwrap();
        let migration = include_str!("../../../migrations/20261018130000_add_season_recap.sql");
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL lock_timeout='2s'")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::raw_sql(migration).execute(&mut *tx).await.unwrap();
        tx.rollback().await.unwrap();
        let absent: Option<String> =
            sqlx::query_scalar("SELECT to_regclass('f_i_season_recap_profiles')::text")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(absent.is_none());
        println!("migration transaction rollback: new tables absent");
        sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        sqlx::query("INSERT INTO f_i_users VALUES($1),($2)")
            .bind(a)
            .bind(b)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::raw_sql("INSERT INTO f_i_matches VALUES (1,2026,'2026-01-01','19:35','成都蓉城','北京国安','3',true),(2,2026,'2026-01-02','19:35','上海申花','成都蓉城','3',true),(3,2026,'2026-01-03','19:35','成都蓉城希拉谷','北京国安','3',true),(4,2026,'2026-01-04','19:35','成都蓉城','上海申花','1',true),(5,2026,'2026-01-05','19:35','成都蓉城','上海申花','3',false),(6,2027,'2027-01-01','19:35','成都蓉城','上海申花','3',true)").execute(&pool).await.unwrap();
        let repo = PostgresRecapRepository::new(pool.clone());
        let initial = repo.load(a, 2026).await.unwrap();
        assert_eq!(initial.matches.len(), 2);
        let saved = save(&repo, a, 2026, &[1, 1, 2], Some(12.25)).await.unwrap();
        let mut read_tx = pool.begin().await.unwrap();
        snapshot(&mut read_tx, a, 2026).await.unwrap();
        let mut update_tx = pool.begin().await.unwrap();
        let read_does_not_block =
            sqlx::query("SELECT match_id FROM f_i_matches WHERE match_id=1 FOR UPDATE NOWAIT")
                .fetch_one(&mut *update_tx)
                .await;
        update_tx.rollback().await.unwrap();
        read_tx.rollback().await.unwrap();
        assert!(
            read_does_not_block.is_ok(),
            "reading recap must not lock scraper match rows"
        );
        let mut locked_tx = pool.begin().await.unwrap();
        snapshot_with_lock(&mut locked_tx, a, 2026, true)
            .await
            .unwrap();
        let mut concurrent_tx = pool.begin().await.unwrap();
        let blocked =
            sqlx::query("SELECT match_id FROM f_i_matches WHERE match_id=1 FOR UPDATE NOWAIT")
                .fetch_one(&mut *concurrent_tx)
                .await
                .unwrap_err();
        assert_eq!(
            blocked.as_database_error().unwrap().code().unwrap(),
            "55P03"
        );
        concurrent_tx.rollback().await.unwrap();
        locked_tx.rollback().await.unwrap();
        assert_eq!(saved.selected, vec![1, 2]);
        assert_eq!(saved.home_one_way_km, Some(12.25));
        assert!(repo.load(b, 2026).await.unwrap().selected.is_empty());
        let kept = save(&repo, a, 2026, &[1, 2], None).await.unwrap();
        assert_eq!(
            kept.home_one_way_km,
            Some(12.25),
            "missing/null distance keeps saved preference"
        );
        save(&repo, b, 2026, &[2], Some(5.0)).await.unwrap();
        save(&repo, a, 2027, &[6], Some(12.25)).await.unwrap();
        for invalid in [3, 4, 5, 6, 99] {
            assert_eq!(
                save(&repo, a, 2026, &[invalid], Some(100.0))
                    .await
                    .unwrap_err(),
                RecapError::InvalidMatch
            );
        }
        assert_eq!(
            repo.load(a, 2026).await.unwrap().home_one_way_km,
            Some(12.25)
        );
        save(&repo, a, 2026, &[], Some(0.0)).await.unwrap();
        assert_eq!(repo.load(b, 2026).await.unwrap().selected, vec![2]);
        assert_eq!(repo.load(a, 2027).await.unwrap().selected, vec![6]);
        for distance in [-1.0, 501.0, f64::NAN, f64::INFINITY] {
            assert!(save(&repo, a, 2026, &[], Some(distance)).await.is_err());
        }
        println!(
            "actual SQL passed: eligible first-team/finished/active/season filter; unique selections; user isolation; per-season replacement; invalid selection rollback; distance constraint; reads do not lock matches; saves hold shared locks; omitted/null distance preserved"
        );
        pool.close().await;
    }
}
