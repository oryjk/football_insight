use async_trait::async_trait;
use sqlx::{PgPool, Row, postgres::PgRow};

use crate::admin::{
    domain::admin_support_bases::{
        AdminSupportBasesError, SupportBaseAudit, SupportBaseInput, SupportBaseMatch,
    },
    ports::admin_support_bases_repository::AdminSupportBasesRepository,
};

pub struct PostgresAdminSupportBasesRepository {
    pool: PgPool,
}

impl PostgresAdminSupportBasesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const SELECT_MATCH: &str = r#"
SELECT m.match_id, m.round_number, TO_CHAR(m.match_date, 'YYYY-MM-DD') AS match_date,
       m.match_time, m.status,
       m.home_team_id, m.home_team_name, m.away_team_id, m.away_team_name,
       (SELECT COUNT(*) FROM f_i_match_support_votes v
         WHERE v.match_id = m.match_id AND v.supported_team_id = m.home_team_id)::bigint AS home_real_count,
       (SELECT COUNT(*) FROM f_i_match_support_votes v
         WHERE v.match_id = m.match_id AND v.supported_team_id = m.away_team_id)::bigint AS away_real_count,
       COALESCE((SELECT b.base_count FROM f_i_match_support_bases b
         WHERE b.match_id = m.match_id AND b.team_id = m.home_team_id), 0)::bigint AS home_base,
       COALESCE((SELECT b.base_count FROM f_i_match_support_bases b
         WHERE b.match_id = m.match_id AND b.team_id = m.away_team_id), 0)::bigint AS away_base
FROM f_i_matches m
"#;

// Beijing calendar days: two weeks back covers just-finished matches, ordered soonest first.
pub fn list_sql() -> String {
    format!(
        "{SELECT_MATCH} WHERE m.source_active = TRUE \
         AND m.match_date >= ((NOW() AT TIME ZONE 'Asia/Shanghai')::date - 14) \
         ORDER BY m.match_date, m.match_time, m.match_id LIMIT $1"
    )
}

pub fn one_sql() -> String {
    format!("{SELECT_MATCH} WHERE m.match_id = $1 AND m.source_active = TRUE")
}

fn map_error(error: sqlx::Error) -> AdminSupportBasesError {
    tracing::error!(%error, "admin support bases persistence failed");
    AdminSupportBasesError::Internal
}

fn read(row: &PgRow) -> Result<SupportBaseMatch, sqlx::Error> {
    Ok(SupportBaseMatch {
        match_id: row.try_get("match_id")?,
        round_number: row.try_get("round_number")?,
        match_date: row.try_get("match_date")?,
        match_time: row.try_get("match_time")?,
        status: row.try_get("status")?,
        home_team_id: row.try_get("home_team_id")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_id: row.try_get("away_team_id")?,
        away_team_name: row.try_get("away_team_name")?,
        home_real_count: row.try_get("home_real_count")?,
        away_real_count: row.try_get("away_real_count")?,
        home_base: row.try_get("home_base")?,
        away_base: row.try_get("away_base")?,
    })
}

#[async_trait]
impl AdminSupportBasesRepository for PostgresAdminSupportBasesRepository {
    async fn list_matches(
        &self,
        limit: i64,
    ) -> Result<Vec<SupportBaseMatch>, AdminSupportBasesError> {
        let rows = sqlx::query(&list_sql())
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;
        rows.iter()
            .map(read)
            .collect::<Result<_, _>>()
            .map_err(map_error)
    }

    async fn set_bases(
        &self,
        match_id: i64,
        input: SupportBaseInput,
        audit: SupportBaseAudit,
    ) -> Result<SupportBaseMatch, AdminSupportBasesError> {
        let mut tx = self.pool.begin().await.map_err(map_error)?;
        let before = sqlx::query(&one_sql())
            .bind(match_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_error)?
            .ok_or(AdminSupportBasesError::NotFound)?;
        let before = read(&before).map_err(map_error)?;
        for (team_id, base) in [
            (before.home_team_id, input.home_base),
            (before.away_team_id, input.away_base),
        ] {
            sqlx::query(
                "INSERT INTO f_i_match_support_bases(match_id, team_id, base_count) VALUES($1,$2,$3) \
                 ON CONFLICT (match_id, team_id) DO UPDATE SET base_count = EXCLUDED.base_count, updated_at = clock_timestamp()",
            )
            .bind(match_id)
            .bind(team_id)
            .bind(base)
            .execute(&mut *tx)
            .await
            .map_err(map_error)?;
        }
        sqlx::query(
            "INSERT INTO f_i_admin_audit_logs(id,admin_user_id,action,target_type,target_id,before_json,after_json) \
             VALUES($1,$2,'support.base.set','match',$3,$4,$5)",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(audit.admin_id)
        .bind(match_id.to_string())
        .bind(serde_json::json!({"home_base": before.home_base, "away_base": before.away_base}))
        .bind(serde_json::json!({"home_base": input.home_base, "away_base": input.away_base}))
        .execute(&mut *tx)
        .await
        .map_err(map_error)?;
        let after = sqlx::query(&one_sql())
            .bind(match_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_error)?;
        let after = read(&after).map_err(map_error)?;
        tx.commit().await.map_err(map_error)?;
        Ok(after)
    }
}
