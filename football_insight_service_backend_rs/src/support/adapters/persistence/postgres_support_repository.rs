use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::support::{
    domain::support::{
        SupportMatchDetail, SupportMatchTeam, SupportTeamSummary, SupportUserContext,
        SupportViewerState, SupportWindowStatus,
    },
    ports::support_repository::SupportRepository,
};

#[derive(Clone)]
pub struct PostgresSupportRepository {
    pool: PgPool,
}

impl PostgresSupportRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct SupportTeamRow {
    team_id: i64,
    team_name: String,
    avatar_storage_url: Option<String>,
    rank_no: Option<i32>,
}

impl From<SupportTeamRow> for SupportTeamSummary {
    fn from(value: SupportTeamRow) -> Self {
        Self {
            team_id: value.team_id,
            team_name: value.team_name,
            avatar_storage_url: value.avatar_storage_url,
            rank_no: value.rank_no,
        }
    }
}

#[derive(Debug, FromRow)]
struct SupportUserContextRow {
    user_id: Uuid,
    favorite_team_id: Option<i64>,
    favorite_team_name: Option<String>,
    favorite_team_avatar_storage_url: Option<String>,
    favorite_team_rank_no: Option<i32>,
}

#[derive(Debug, FromRow)]
struct MatchIdRow {
    match_id: i64,
}

#[derive(Debug, FromRow)]
struct SupportMatchDetailRow {
    match_id: i64,
    season: i32,
    round_number: i32,
    match_date: String,
    match_time: String,
    status: String,
    kickoff_at: DateTime<Utc>,
    home_team_id: i64,
    home_team_name: String,
    home_team_avatar_storage_url: Option<String>,
    home_score: String,
    away_team_id: i64,
    away_team_name: String,
    away_team_avatar_storage_url: Option<String>,
    away_score: String,
    home_support_count: i64,
    away_support_count: i64,
    home_support_share_pct: i32,
    away_support_share_pct: i32,
    total_support_count: i64,
    home_season_support_rank: Option<i32>,
    away_season_support_rank: Option<i32>,
    viewer_favorite_team_id: Option<i64>,
    viewer_supported_team_id: Option<i64>,
}

impl SupportMatchDetailRow {
    fn into_domain(self, viewer_user_id: Option<Uuid>) -> SupportMatchDetail {
        SupportMatchDetail {
            match_id: self.match_id,
            season: self.season,
            round_number: self.round_number,
            match_date: self.match_date,
            match_time: self.match_time,
            status: self.status,
            kickoff_at: self.kickoff_at,
            support_window_status: SupportWindowStatus::Locked,
            countdown_seconds: 0,
            total_support_count: self.total_support_count,
            home_team: SupportMatchTeam {
                team_id: self.home_team_id,
                team_name: self.home_team_name,
                avatar_storage_url: self.home_team_avatar_storage_url,
                score: self.home_score,
                support_count: self.home_support_count,
                support_share_pct: self.home_support_share_pct,
                season_support_rank: self.home_season_support_rank,
            },
            away_team: SupportMatchTeam {
                team_id: self.away_team_id,
                team_name: self.away_team_name,
                avatar_storage_url: self.away_team_avatar_storage_url,
                score: self.away_score,
                support_count: self.away_support_count,
                support_share_pct: self.away_support_share_pct,
                season_support_rank: self.away_season_support_rank,
            },
            viewer: SupportViewerState {
                user_id: viewer_user_id,
                favorite_team_id: self.viewer_favorite_team_id,
                supported_team_id: self.viewer_supported_team_id,
                has_supported: self.viewer_supported_team_id.is_some(),
                can_support: false,
            },
        }
    }
}

#[async_trait]
impl SupportRepository for PostgresSupportRepository {
    async fn list_teams(&self) -> anyhow::Result<Vec<SupportTeamSummary>> {
        let rows = sqlx::query_as::<_, SupportTeamRow>(
            r#"
            WITH latest_standings AS (
                SELECT DISTINCT ON (team_id)
                       team_id,
                       rank_no
                  FROM f_i_standings
                 ORDER BY team_id, snapshot_at DESC
            )
            SELECT t.team_id,
                   t.team_name,
                   t.avatar_storage_url,
                   ls.rank_no
              FROM f_i_teams t
              LEFT JOIN latest_standings ls ON ls.team_id = t.team_id
             ORDER BY COALESCE(ls.rank_no, 9999), t.team_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_user_context(&self, user_id: Uuid) -> anyhow::Result<SupportUserContext> {
        let row = sqlx::query_as::<_, SupportUserContextRow>(
            r#"
            WITH latest_standings AS (
                SELECT DISTINCT ON (team_id)
                       team_id,
                       rank_no
                  FROM f_i_standings
                 ORDER BY team_id, snapshot_at DESC
            )
            SELECT u.id AS user_id,
                   t.team_id AS favorite_team_id,
                   t.team_name AS favorite_team_name,
                   t.avatar_storage_url AS favorite_team_avatar_storage_url,
                   ls.rank_no AS favorite_team_rank_no
              FROM f_i_users u
              LEFT JOIN f_i_teams t ON t.team_id = u.favorite_team_id
              LEFT JOIN latest_standings ls ON ls.team_id = t.team_id
             WHERE u.id = $1
             LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("user not found"))?;

        Ok(SupportUserContext {
            user_id: row.user_id,
            favorite_team: row.favorite_team_id.map(|team_id| SupportTeamSummary {
                team_id,
                team_name: row.favorite_team_name.unwrap_or_default(),
                avatar_storage_url: row.favorite_team_avatar_storage_url,
                rank_no: row.favorite_team_rank_no,
            }),
        })
    }

    async fn set_favorite_team(
        &self,
        user_id: Uuid,
        team_id: i64,
    ) -> anyhow::Result<SupportTeamSummary> {
        let row = sqlx::query_as::<_, SupportTeamRow>(
            r#"
            WITH latest_standings AS (
                SELECT DISTINCT ON (team_id)
                       team_id,
                       rank_no
                  FROM f_i_standings
                 ORDER BY team_id, snapshot_at DESC
            ),
            candidate AS (
                SELECT t.team_id,
                       t.team_name,
                       t.avatar_storage_url,
                       ls.rank_no
                  FROM f_i_teams t
                  LEFT JOIN latest_standings ls ON ls.team_id = t.team_id
                 WHERE t.team_id = $2
            )
            UPDATE f_i_users u
               SET favorite_team_id = candidate.team_id,
                   updated_at = NOW()
              FROM candidate
             WHERE u.id = $1
         RETURNING candidate.team_id,
                   candidate.team_name,
                   candidate.avatar_storage_url,
                   candidate.rank_no
            "#,
        )
        .bind(user_id)
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(Into::into)
            .ok_or_else(|| anyhow::anyhow!("favorite team not found"))
    }

    async fn find_matches_for_team(
        &self,
        team_id: i64,
        viewer_user_id: Option<Uuid>,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<SupportMatchDetail>> {
        let match_ids = sqlx::query_as::<_, MatchIdRow>(
            r#"
            SELECT m.match_id
              FROM f_i_matches m
             WHERE (m.home_team_id = $1 OR m.away_team_id = $1)
               AND ((m.match_date::timestamp + m.match_time::time) AT TIME ZONE 'Asia/Shanghai') > $2
             ORDER BY m.match_date ASC, m.match_time ASC, m.match_id ASC
             LIMIT 3
            "#,
        )
        .bind(team_id)
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        let mut items = Vec::with_capacity(match_ids.len());
        for row in match_ids {
            if let Some(detail) = self.find_match_detail(row.match_id, viewer_user_id).await? {
                items.push(detail);
            }
        }

        Ok(items)
    }

    async fn find_match_detail(
        &self,
        match_id: i64,
        viewer_user_id: Option<Uuid>,
    ) -> anyhow::Result<Option<SupportMatchDetail>> {
        let row = sqlx::query_as::<_, SupportMatchDetailRow>(
            r#"
            WITH selected_match AS (
                SELECT m.match_id,
                       m.season,
                       m.round_number,
                       TO_CHAR(m.match_date, 'YYYY-MM-DD') AS match_date,
                       m.match_time,
                       m.status,
                       ((m.match_date::timestamp + m.match_time::time) AT TIME ZONE 'Asia/Shanghai') AS kickoff_at,
                       m.home_team_id,
                       m.home_team_name,
                       ht.avatar_storage_url AS home_team_avatar_storage_url,
                       m.home_score,
                       m.away_team_id,
                       m.away_team_name,
                       at.avatar_storage_url AS away_team_avatar_storage_url,
                       m.away_score
                  FROM f_i_matches m
                  LEFT JOIN f_i_teams ht ON ht.team_id = m.home_team_id
                  LEFT JOIN f_i_teams at ON at.team_id = m.away_team_id
                 WHERE m.match_id = $1
            ),
            match_support_totals AS (
                SELECT v.supported_team_id,
                       COUNT(*)::BIGINT AS support_count
                  FROM f_i_match_support_votes v
                 WHERE v.match_id = $1
                 GROUP BY v.supported_team_id
            ),
            season_support_totals AS (
                SELECT v.supported_team_id,
                       DENSE_RANK() OVER (ORDER BY COUNT(*) DESC, v.supported_team_id ASC)::INT AS season_support_rank
                  FROM f_i_match_support_votes v
                  JOIN f_i_matches m ON m.match_id = v.match_id
                  JOIN selected_match sm ON sm.season = m.season
                 GROUP BY v.supported_team_id
            ),
            viewer_context AS (
                SELECT favorite_team_id
                  FROM f_i_users
                 WHERE id = $2
            ),
            viewer_vote AS (
                SELECT supported_team_id
                  FROM f_i_match_support_votes
                 WHERE match_id = $1
                   AND user_id = $2
                 LIMIT 1
            )
            SELECT sm.match_id,
                   sm.season,
                   sm.round_number,
                   sm.match_date,
                   sm.match_time,
                   sm.status,
                   sm.kickoff_at,
                   sm.home_team_id,
                   sm.home_team_name,
                   sm.home_team_avatar_storage_url,
                   sm.home_score,
                   sm.away_team_id,
                   sm.away_team_name,
                   sm.away_team_avatar_storage_url,
                   sm.away_score,
                   COALESCE(home_totals.support_count, 0) AS home_support_count,
                   COALESCE(away_totals.support_count, 0) AS away_support_count,
                   CASE
                       WHEN (COALESCE(home_totals.support_count, 0) + COALESCE(away_totals.support_count, 0)) = 0
                           THEN 0
                       ELSE ROUND(
                           (COALESCE(home_totals.support_count, 0) * 100.0)
                           / (COALESCE(home_totals.support_count, 0) + COALESCE(away_totals.support_count, 0))
                       )::INT
                   END AS home_support_share_pct,
                   CASE
                       WHEN (COALESCE(home_totals.support_count, 0) + COALESCE(away_totals.support_count, 0)) = 0
                           THEN 0
                       ELSE ROUND(
                           (COALESCE(away_totals.support_count, 0) * 100.0)
                           / (COALESCE(home_totals.support_count, 0) + COALESCE(away_totals.support_count, 0))
                       )::INT
                   END AS away_support_share_pct,
                   (COALESCE(home_totals.support_count, 0) + COALESCE(away_totals.support_count, 0)) AS total_support_count,
                   home_rank.season_support_rank AS home_season_support_rank,
                   away_rank.season_support_rank AS away_season_support_rank,
                   vc.favorite_team_id AS viewer_favorite_team_id,
                   vv.supported_team_id AS viewer_supported_team_id
              FROM selected_match sm
              LEFT JOIN match_support_totals home_totals ON home_totals.supported_team_id = sm.home_team_id
              LEFT JOIN match_support_totals away_totals ON away_totals.supported_team_id = sm.away_team_id
              LEFT JOIN season_support_totals home_rank ON home_rank.supported_team_id = sm.home_team_id
              LEFT JOIN season_support_totals away_rank ON away_rank.supported_team_id = sm.away_team_id
              LEFT JOIN viewer_context vc ON TRUE
              LEFT JOIN viewer_vote vv ON TRUE
            "#,
        )
        .bind(match_id)
        .bind(viewer_user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|item| item.into_domain(viewer_user_id)))
    }

    async fn create_vote(
        &self,
        user_id: Uuid,
        match_id: i64,
        supported_team_id: i64,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            r#"
            INSERT INTO f_i_match_support_votes (
                id,
                match_id,
                user_id,
                supported_team_id
            ) VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(match_id)
        .bind(user_id)
        .bind(supported_team_id)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23505") => {
                anyhow::bail!("you have already supported this match")
            }
            Err(error) => Err(error.into()),
        }
    }
}
