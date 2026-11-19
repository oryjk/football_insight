use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

use crate::{
    auth::domain::user::sanitize_public_display_name,
    director::{
        domain::{
            DirectorAuthor, DirectorComment, DirectorError, DirectorPlan, DirectorTeam,
            DirectorWindow, PlanPlayerItem, PoolCandidate, ReactionSummary, SquadPlayer,
        },
        ports::{DirectorRepository, DirectorUserProfile, ExistingPlanCodes, UpsertPlanRecord},
    },
};

pub struct PostgresDirectorRepository {
    pool: PgPool,
}

impl PostgresDirectorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_db_error(error: sqlx::Error) -> DirectorError {
    if let Some(db) = error.as_database_error() {
        if db.code().as_deref() == Some("42P01") {
            tracing::warn!(error = %error, "director table missing (42P01), returning 503");
            return DirectorError::Unavailable;
        }
        if db.code().as_deref() == Some("23505")
            && db
                .constraint()
                .is_some_and(|c| c.contains("share_code"))
        {
            return DirectorError::ShareCodeTaken;
        }
    }
    tracing::warn!(error = %error, "director repository error");
    DirectorError::Unavailable
}

const WINDOW_COLUMNS: &str = "id, code, name, opens_at, closes_at";

const POSITION_ORDER: &str =
    "CASE sp.position WHEN 'FW' THEN 1 WHEN 'MF' THEN 2 WHEN 'DF' THEN 3 WHEN 'GK' THEN 4 ELSE 5 END";

const PLAN_SELECT: &str = "SELECT p.id, p.window_id, w.code AS window_code, w.name AS window_name,
       p.team_id, t.team_name, p.share_code, p.circle_code, p.parent_share_code,
       p.user_id, u.display_name, u.avatar_url AS author_avatar_url,
       p.foreign_stays, p.foreign_leaves, p.foreign_ins, p.domestic_ins, p.domestic_outs,
       p.style_tag, p.manifesto, p.updated_at,
       COALESCE(rx.likes_count, 0) AS likes_count,
       COALESCE(rx.dislikes_count, 0) AS dislikes_count,
       COALESCE(rx.my_reaction, 0)::SMALLINT AS my_reaction,
       COALESCE(cm.comments_count, 0) AS comments_count
FROM f_i_director_plans p
JOIN f_i_transfer_windows w ON w.id = p.window_id
JOIN f_i_teams t ON t.team_id = p.team_id
JOIN f_i_users u ON u.id = p.user_id AND u.status = 'active'
LEFT JOIN LATERAL (
    SELECT COUNT(*) FILTER (WHERE r.value = 1) AS likes_count,
           COUNT(*) FILTER (WHERE r.value = -1) AS dislikes_count,
           COALESCE(MAX(r.value) FILTER (WHERE r.user_id = $1), 0)::SMALLINT AS my_reaction
    FROM f_i_director_plan_reactions r
    JOIN f_i_users ru ON ru.id = r.user_id AND ru.status = 'active'
    WHERE r.plan_id = p.id
) rx ON TRUE
LEFT JOIN LATERAL (
    SELECT COUNT(*) AS comments_count
    FROM f_i_director_plan_comments c
    JOIN f_i_users cu ON cu.id = c.user_id AND cu.status = 'active'
    WHERE c.plan_id = p.id
) cm ON TRUE";

#[derive(FromRow)]
struct WindowRow {
    id: i64,
    code: String,
    name: String,
    opens_at: DateTime<Utc>,
    closes_at: DateTime<Utc>,
}

impl From<WindowRow> for DirectorWindow {
    fn from(r: WindowRow) -> Self {
        Self {
            id: r.id,
            code: r.code,
            name: r.name,
            opens_at: r.opens_at,
            closes_at: r.closes_at,
        }
    }
}

#[derive(FromRow)]
struct SquadPlayerRow {
    player_id: i64,
    team_id: i64,
    team_name: String,
    player_name: String,
    shirt_number: Option<i32>,
    position: Option<String>,
    country: Option<String>,
    avatar_storage_url: Option<String>,
}

impl From<SquadPlayerRow> for SquadPlayer {
    fn from(r: SquadPlayerRow) -> Self {
        Self {
            player_id: r.player_id,
            team_id: r.team_id,
            team_name: r.team_name,
            player_name: r.player_name,
            shirt_number: r.shirt_number,
            position: r.position,
            country: r.country,
            avatar_url: r.avatar_storage_url,
        }
    }
}

#[derive(FromRow)]
struct PoolCandidateRow {
    code: String,
    player_name: String,
    position: String,
    country: String,
    is_foreign: bool,
    league_tag: String,
    current_club: String,
    tags: Json<Vec<String>>,
    avatar_url: Option<String>,
}

impl From<PoolCandidateRow> for PoolCandidate {
    fn from(r: PoolCandidateRow) -> Self {
        Self {
            code: r.code,
            player_name: r.player_name,
            position: r.position,
            country: r.country,
            is_foreign: r.is_foreign,
            league_tag: r.league_tag,
            current_club: r.current_club,
            tags: r.tags.0,
            avatar_url: r.avatar_url,
        }
    }
}

#[derive(FromRow)]
struct PlanRow {
    id: Uuid,
    window_id: i64,
    window_code: String,
    window_name: String,
    team_id: i64,
    team_name: String,
    share_code: String,
    circle_code: String,
    parent_share_code: Option<String>,
    user_id: Uuid,
    display_name: Option<String>,
    author_avatar_url: Option<String>,
    foreign_stays: Json<Vec<PlanPlayerItem>>,
    foreign_leaves: Json<Vec<PlanPlayerItem>>,
    foreign_ins: Json<Vec<PlanPlayerItem>>,
    domestic_ins: Json<Vec<PlanPlayerItem>>,
    domestic_outs: Json<Vec<PlanPlayerItem>>,
    style_tag: String,
    manifesto: String,
    updated_at: DateTime<Utc>,
    likes_count: i64,
    dislikes_count: i64,
    my_reaction: i16,
    comments_count: i64,
}

impl PlanRow {
    fn into_domain(self, viewer: Option<Uuid>) -> DirectorPlan {
        DirectorPlan {
            id: self.id,
            window_id: self.window_id,
            window_code: self.window_code,
            window_name: self.window_name,
            team_id: self.team_id,
            team_name: self.team_name,
            share_code: self.share_code,
            circle_code: self.circle_code,
            parent_share_code: self.parent_share_code,
            author: DirectorAuthor {
                user_id: self.user_id,
                nickname: sanitize_public_display_name(self.display_name.as_deref()),
                avatar_url: self.author_avatar_url,
            },
            is_mine: viewer == Some(self.user_id),
            foreign_stays: self.foreign_stays.0,
            foreign_leaves: self.foreign_leaves.0,
            foreign_ins: self.foreign_ins.0,
            domestic_ins: self.domestic_ins.0,
            domestic_outs: self.domestic_outs.0,
            style_tag: self.style_tag,
            manifesto: self.manifesto,
            likes_count: self.likes_count,
            dislikes_count: self.dislikes_count,
            my_reaction: self.my_reaction,
            comments_count: self.comments_count,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
struct CommentRow {
    id: Uuid,
    user_id: Uuid,
    display_name: Option<String>,
    avatar_url: Option<String>,
    content: String,
    created_at: DateTime<Utc>,
}

impl From<CommentRow> for DirectorComment {
    fn from(r: CommentRow) -> Self {
        Self {
            id: r.id,
            author: DirectorAuthor {
                user_id: r.user_id,
                nickname: sanitize_public_display_name(r.display_name.as_deref()),
                avatar_url: r.avatar_url,
            },
            content: r.content,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl DirectorRepository for PostgresDirectorRepository {
    async fn current_window(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Option<DirectorWindow>, DirectorError> {
        let open = sqlx::query_as::<_, WindowRow>(&format!(
            "SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows
             WHERE opens_at <= $1 AND $1 < closes_at
             ORDER BY opens_at DESC LIMIT 1"
        ))
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        if let Some(window) = open {
            return Ok(Some(window.into()));
        }
        sqlx::query_as::<_, WindowRow>(&format!(
            "SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows
             WHERE closes_at <= $1
             ORDER BY closes_at DESC LIMIT 1"
        ))
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    async fn find_team(&self, team_id: i64) -> Result<Option<DirectorTeam>, DirectorError> {
        sqlx::query_as::<_, (i64, String, Option<String>)>(
            "SELECT team_id, team_name, avatar_storage_url FROM f_i_teams WHERE team_id = $1",
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|(team_id, team_name, logo_url)| DirectorTeam {
                team_id,
                team_name,
                logo_url,
            })
        })
        .map_err(map_db_error)
    }

    async fn team_active_squad(&self, team_id: i64) -> Result<Vec<SquadPlayer>, DirectorError> {
        sqlx::query_as::<_, SquadPlayerRow>(&format!(
            "SELECT sp.player_id, sp.team_id, t.team_name, sp.player_name,
                    sp.shirt_number, sp.position, sp.country, sp.avatar_storage_url
             FROM f_i_squad_players sp
             JOIN f_i_teams t ON t.team_id = sp.team_id
             WHERE sp.team_id = $1 AND sp.active = TRUE
             ORDER BY {POSITION_ORDER}, sp.shirt_number NULLS LAST, sp.player_id"
        ))
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(map_db_error)
    }

    async fn other_teams_active_squad(
        &self,
        exclude_team_id: i64,
        keyword: Option<&str>,
        limit: i64,
    ) -> Result<Vec<SquadPlayer>, DirectorError> {
        let pattern = keyword.map(|kw| format!("%{kw}%"));
        sqlx::query_as::<_, SquadPlayerRow>(&format!(
            "SELECT sp.player_id, sp.team_id, t.team_name, sp.player_name,
                    sp.shirt_number, sp.position, sp.country, sp.avatar_storage_url
             FROM f_i_squad_players sp
             JOIN f_i_teams t ON t.team_id = sp.team_id
             WHERE sp.active = TRUE
               AND sp.team_id <> $1
               AND NOT EXISTS (
                   SELECT 1 FROM f_i_squad_players own
                   WHERE own.team_id = $1 AND own.player_id = sp.player_id AND own.active = TRUE
               )
               AND ($2::TEXT IS NULL OR sp.player_name ILIKE $2 OR t.team_name ILIKE $2)
             ORDER BY {POSITION_ORDER}, t.team_name, sp.player_id
             LIMIT $3"
        ))
        .bind(exclude_team_id)
        .bind(pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(map_db_error)
    }

    async fn find_active_squad_player(
        &self,
        player_id: i64,
        exclude_team_id: i64,
    ) -> Result<Option<SquadPlayer>, DirectorError> {
        sqlx::query_as::<_, SquadPlayerRow>(
            "SELECT sp.player_id, sp.team_id, t.team_name, sp.player_name,
                    sp.shirt_number, sp.position, sp.country, sp.avatar_storage_url
             FROM f_i_squad_players sp
             JOIN f_i_teams t ON t.team_id = sp.team_id
             WHERE sp.player_id = $1
               AND sp.active = TRUE
               AND sp.team_id <> $2
               AND NOT EXISTS (
                   SELECT 1 FROM f_i_squad_players own
                   WHERE own.team_id = $2 AND own.player_id = sp.player_id AND own.active = TRUE
               )
             ORDER BY sp.last_season DESC
             LIMIT 1",
        )
        .bind(player_id)
        .bind(exclude_team_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    async fn list_pool_candidates(
        &self,
        team_id: i64,
        is_foreign: bool,
        league: &str,
        keyword: Option<&str>,
        limit: i64,
    ) -> Result<Vec<PoolCandidate>, DirectorError> {
        let pattern = keyword.map(|kw| format!("%{kw}%"));
        sqlx::query_as::<_, PoolCandidateRow>(
            "SELECT code, player_name, position, country, is_foreign,
                    league_tag, current_club, tags, avatar_url
             FROM f_i_director_candidates
             WHERE active = TRUE
               AND is_foreign = $2
               AND (
                   $3 = 'ALL'
                   OR ($3 = 'K' AND UPPER(league_tag) IN ('K1', 'K2'))
                   OR ($3 = 'OTHER' AND UPPER(league_tag) NOT IN ('K1', 'K2', 'J1', 'CSL', 'FREE'))
                   OR UPPER(league_tag) = $3
               )
               AND ($4::TEXT IS NULL OR player_name ILIKE $4 OR current_club ILIKE $4)
             ORDER BY (target_team_id = $1) DESC NULLS LAST, sort_order ASC, id ASC
             LIMIT $5",
        )
        .bind(team_id)
        .bind(is_foreign)
        .bind(league)
        .bind(pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(map_db_error)
    }

    async fn find_pool_candidate(
        &self,
        code: &str,
    ) -> Result<Option<PoolCandidate>, DirectorError> {
        sqlx::query_as::<_, PoolCandidateRow>(
            "SELECT code, player_name, position, country, is_foreign,
                    league_tag, current_club, tags, avatar_url
             FROM f_i_director_candidates
             WHERE code = $1 AND active = TRUE
             LIMIT 1",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_db_error)
    }

    async fn count_team_plans(&self, window_id: i64, team_id: i64) -> Result<i64, DirectorError> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM f_i_director_plans p
             JOIN f_i_users u ON u.id = p.user_id AND u.status = 'active'
             WHERE p.window_id = $1 AND p.team_id = $2",
        )
        .bind(window_id)
        .bind(team_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_user_plan_codes(
        &self,
        window_id: i64,
        team_id: i64,
        user_id: Uuid,
    ) -> Result<Option<ExistingPlanCodes>, DirectorError> {
        sqlx::query_as::<_, (String, String, Option<String>)>(
            "SELECT share_code, circle_code, parent_share_code
             FROM f_i_director_plans
             WHERE window_id = $1 AND team_id = $2 AND user_id = $3
             LIMIT 1",
        )
        .bind(window_id)
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(
                |(share_code, circle_code, parent_share_code)| ExistingPlanCodes {
                    share_code,
                    circle_code,
                    parent_share_code,
                },
            )
        })
        .map_err(map_db_error)
    }

    async fn circle_exists(
        &self,
        window_id: i64,
        team_id: i64,
        circle_code: &str,
    ) -> Result<bool, DirectorError> {
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM f_i_director_plans
                WHERE window_id = $1 AND team_id = $2 AND circle_code = $3
            )",
        )
        .bind(window_id)
        .bind(team_id)
        .bind(circle_code)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_circle_code_by_share_code(
        &self,
        window_id: i64,
        team_id: i64,
        share_code: &str,
    ) -> Result<Option<String>, DirectorError> {
        sqlx::query_scalar::<_, String>(
            "SELECT circle_code FROM f_i_director_plans
             WHERE window_id = $1 AND team_id = $2 AND share_code = $3
             LIMIT 1",
        )
        .bind(window_id)
        .bind(team_id)
        .bind(share_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn upsert_plan(&self, record: &UpsertPlanRecord) -> Result<String, DirectorError> {
        sqlx::query_scalar::<_, String>(
            "INSERT INTO f_i_director_plans (
                 window_id, team_id, user_id, share_code, circle_code, parent_share_code,
                 foreign_stays, foreign_leaves, foreign_ins, domestic_ins, domestic_outs,
                 style_tag, manifesto
             )
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT (window_id, team_id, user_id) DO UPDATE SET
                 foreign_stays = EXCLUDED.foreign_stays,
                 foreign_leaves = EXCLUDED.foreign_leaves,
                 foreign_ins = EXCLUDED.foreign_ins,
                 domestic_ins = EXCLUDED.domestic_ins,
                 domestic_outs = EXCLUDED.domestic_outs,
                 style_tag = EXCLUDED.style_tag,
                 manifesto = EXCLUDED.manifesto,
                 updated_at = NOW()
             RETURNING share_code",
        )
        .bind(record.window_id)
        .bind(record.team_id)
        .bind(record.user_id)
        .bind(&record.share_code)
        .bind(&record.circle_code)
        .bind(&record.parent_share_code)
        .bind(Json(&record.foreign_stays))
        .bind(Json(&record.foreign_leaves))
        .bind(Json(&record.foreign_ins))
        .bind(Json(&record.domestic_ins))
        .bind(Json(&record.domestic_outs))
        .bind(&record.style_tag)
        .bind(&record.manifesto)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn find_plan_by_share_code(
        &self,
        share_code: &str,
        viewer_user_id: Option<Uuid>,
    ) -> Result<Option<DirectorPlan>, DirectorError> {
        sqlx::query_as::<_, PlanRow>(&format!(
            "{PLAN_SELECT} WHERE p.share_code = $2 LIMIT 1"
        ))
        .bind(viewer_user_id)
        .bind(share_code)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(|r| r.into_domain(viewer_user_id)))
        .map_err(map_db_error)
    }

    async fn find_plan_by_user(
        &self,
        window_id: i64,
        team_id: i64,
        user_id: Uuid,
        viewer_user_id: Option<Uuid>,
    ) -> Result<Option<DirectorPlan>, DirectorError> {
        sqlx::query_as::<_, PlanRow>(&format!(
            "{PLAN_SELECT} WHERE p.window_id = $2 AND p.team_id = $3 AND p.user_id = $4 LIMIT 1"
        ))
        .bind(viewer_user_id)
        .bind(window_id)
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(|r| r.into_domain(viewer_user_id)))
        .map_err(map_db_error)
    }

    async fn list_plan_comments(
        &self,
        plan_id: Uuid,
        limit: i64,
    ) -> Result<Vec<DirectorComment>, DirectorError> {
        sqlx::query_as::<_, CommentRow>(
            "SELECT c.id, c.user_id, u.display_name, u.avatar_url, c.content, c.created_at
             FROM f_i_director_plan_comments c
             JOIN f_i_users u ON u.id = c.user_id AND u.status = 'active'
             WHERE c.plan_id = $1
             ORDER BY c.created_at DESC
             LIMIT $2",
        )
        .bind(plan_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(map_db_error)
    }

    async fn list_circle_plans(
        &self,
        circle_code: &str,
        viewer_user_id: Option<Uuid>,
    ) -> Result<Vec<DirectorPlan>, DirectorError> {
        sqlx::query_as::<_, PlanRow>(&format!(
            "{PLAN_SELECT} WHERE p.circle_code = $2 ORDER BY p.updated_at DESC"
        ))
        .bind(viewer_user_id)
        .bind(circle_code)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|r| r.into_domain(viewer_user_id))
                .collect()
        })
        .map_err(map_db_error)
    }

    async fn set_plan_reaction(
        &self,
        share_code: &str,
        user_id: Uuid,
        value: i16,
    ) -> Result<ReactionSummary, DirectorError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;
        let plan_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM f_i_director_plans WHERE share_code = $1 LIMIT 1",
        )
        .bind(share_code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DirectorError::NotFound("方案不存在".into()))?;

        if value == 0 {
            sqlx::query(
                "DELETE FROM f_i_director_plan_reactions WHERE plan_id = $1 AND user_id = $2",
            )
            .bind(plan_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
        } else {
            sqlx::query(
                "INSERT INTO f_i_director_plan_reactions (plan_id, user_id, value)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (plan_id, user_id) DO UPDATE SET
                     value = EXCLUDED.value,
                     updated_at = NOW()",
            )
            .bind(plan_id)
            .bind(user_id)
            .bind(value)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
        }

        let (likes_count, dislikes_count, my_reaction) = sqlx::query_as::<_, (i64, i64, i16)>(
            "SELECT COUNT(*) FILTER (WHERE r.value = 1),
                    COUNT(*) FILTER (WHERE r.value = -1),
                    COALESCE(MAX(r.value) FILTER (WHERE r.user_id = $2), 0)::SMALLINT
             FROM f_i_director_plan_reactions r
             JOIN f_i_users u ON u.id = r.user_id AND u.status = 'active'
             WHERE r.plan_id = $1",
        )
        .bind(plan_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

        tx.commit().await.map_err(map_db_error)?;
        Ok(ReactionSummary {
            likes_count,
            dislikes_count,
            my_reaction,
        })
    }

    async fn insert_plan_comment(
        &self,
        share_code: &str,
        user_id: Uuid,
        content: &str,
    ) -> Result<DirectorComment, DirectorError> {
        let plan_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM f_i_director_plans WHERE share_code = $1 LIMIT 1",
        )
        .bind(share_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DirectorError::NotFound("方案不存在".into()))?;

        let row = sqlx::query_as::<_, CommentRow>(
            "WITH inserted AS (
                 INSERT INTO f_i_director_plan_comments (plan_id, user_id, content)
                 VALUES ($1, $2, $3)
                 RETURNING id, user_id, content, created_at
             )
             SELECT i.id, i.user_id, u.display_name, u.avatar_url, i.content, i.created_at
             FROM inserted i
             JOIN f_i_users u ON u.id = i.user_id",
        )
        .bind(plan_id)
        .bind(user_id)
        .bind(content)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(row.into())
    }

    async fn user_profile(
        &self,
        user_id: Uuid,
    ) -> Result<Option<DirectorUserProfile>, DirectorError> {
        sqlx::query_as::<_, (String, Option<i64>, Option<String>)>(
            "SELECT status, favorite_team_id, wx_open_id FROM f_i_users WHERE id = $1 LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|(status, favorite_team_id, wx_open_id)| DirectorUserProfile {
                active: status == "active",
                favorite_team_id,
                wx_open_id,
            })
        })
        .map_err(map_db_error)
    }
}
