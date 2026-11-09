use crate::lineup::{
    domain::{
        Lineup, LineupDraft, LineupReadError, LineupSlot, PlayerClub, Position, PublishedLineup,
        SquadPlayer, SquadTeam,
    },
    ports::{LineupRepository, LineupUserAccess, SquadRepository, StoreError},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

pub struct PostgresSquadRepository {
    pool: PgPool,
}

impl PostgresSquadRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn read_error(e: sqlx::Error) -> LineupReadError {
    let missing = e
        .as_database_error()
        .is_some_and(|d| d.code().as_deref() == Some("42P01"));
    tracing::warn!(missing_table = missing, "lineup squad read unavailable");
    LineupReadError::Unavailable
}

#[async_trait]
impl SquadRepository for PostgresSquadRepository {
    async fn find_team(&self, team_id: i64) -> Result<Option<SquadTeam>, LineupReadError> {
        sqlx::query_as::<_, (i64, String, Option<String>)>(
            "SELECT team_id, team_name, avatar_storage_url FROM f_i_teams WHERE team_id = $1",
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|(team_id, team_name, avatar_url)| SquadTeam {
                team_id,
                team_name,
                avatar_url,
            })
        })
        .map_err(read_error)
    }

    async fn list_squad(&self, team_id: i64) -> Result<Vec<SquadPlayer>, LineupReadError> {
        type Row = (
            i64,
            String,
            Option<i32>,
            Option<String>,
            Option<String>,
            i32,
            bool,
        );
        sqlx::query_as::<_, Row>(
            "SELECT player_id, player_name, shirt_number, position, avatar_storage_url, last_season, active
             FROM f_i_squad_players WHERE team_id = $1",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(
                    |(player_id, name, shirt_number, position, avatar_url, last_season, active)| {
                        SquadPlayer {
                            player_id,
                            name,
                            shirt_number,
                            position: position.as_deref().and_then(Position::parse),
                            avatar_url,
                            last_season,
                            active,
                            club: None,
                        }
                    },
                )
                .collect()
        })
        .map_err(read_error)
    }

    async fn search_players(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<SquadPlayer>, LineupReadError> {
        // strpos 而不是 LIKE：搜索词里的 % 和 _ 不会被当成通配符。
        sqlx::query_as::<_, ClubPlayerRow>(&format!(
            "SELECT * FROM ({CLUB_PLAYERS} WHERE strpos(s.player_name, $1) > 0 {CLUB_PLAYER_ORDER}) p
             ORDER BY (p.player_name = $1) DESC, (strpos(p.player_name, $1) = 1) DESC,
                      p.active DESC, p.last_season DESC, p.player_id
             LIMIT $2"
        ))
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(read_error)
    }

    async fn find_players(&self, player_ids: &[i64]) -> Result<Vec<SquadPlayer>, LineupReadError> {
        sqlx::query_as::<_, ClubPlayerRow>(&format!(
            "{CLUB_PLAYERS} WHERE s.player_id = ANY($1) {CLUB_PLAYER_ORDER}"
        ))
        .bind(player_ids)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(read_error)
    }
}

/// 一名球员可能在多支球队的名单里（转会、前队员）：每人取一行，现役优先，其次最后在队的赛季。
const CLUB_PLAYERS: &str = "SELECT DISTINCT ON (s.player_id)
        s.player_id, s.player_name, s.shirt_number, s.position, s.avatar_storage_url,
        s.last_season, s.active, s.team_id, COALESCE(t.team_name, '') AS team_name
    FROM f_i_squad_players s LEFT JOIN f_i_teams t ON t.team_id = s.team_id";
const CLUB_PLAYER_ORDER: &str = "ORDER BY s.player_id, s.active DESC, s.last_season DESC";

#[derive(sqlx::FromRow)]
struct ClubPlayerRow {
    player_id: i64,
    player_name: String,
    shirt_number: Option<i32>,
    position: Option<String>,
    avatar_storage_url: Option<String>,
    last_season: i32,
    active: bool,
    team_id: i64,
    team_name: String,
}

impl From<ClubPlayerRow> for SquadPlayer {
    fn from(row: ClubPlayerRow) -> Self {
        Self {
            player_id: row.player_id,
            name: row.player_name,
            shirt_number: row.shirt_number,
            position: row.position.as_deref().and_then(Position::parse),
            avatar_url: row.avatar_storage_url,
            last_season: row.last_season,
            active: row.active,
            club: Some(PlayerClub {
                team_id: row.team_id,
                team_name: row.team_name,
            }),
        }
    }
}

pub struct PostgresLineupRepository {
    pool: PgPool,
}

impl PostgresLineupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LineupUserAccess for PostgresLineupRepository {
    async fn is_active(&self, user_id: Uuid) -> Result<bool, StoreError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM f_i_users WHERE id = $1 AND status = 'active')",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(store_error)
    }
}

fn store_error(e: sqlx::Error) -> StoreError {
    if let Some(db) = e.as_database_error()
        && db.code().as_deref() == Some("23505")
        && db.constraint() == Some("uq_f_i_lineups_share_code")
    {
        return StoreError::ShareCodeTaken;
    }
    tracing::warn!("lineup store unavailable");
    StoreError::Unavailable
}

type LineupRow = (
    String,
    Json<Vec<StoredSlot>>,
    String,
    chrono::DateTime<chrono::Utc>,
);

#[derive(Serialize, Deserialize)]
struct StoredSlot {
    slot: u8,
    player_id: Option<i64>,
    x: f64,
    y: f64,
}

impl From<StoredSlot> for LineupSlot {
    fn from(slot: StoredSlot) -> Self {
        Self {
            slot: slot.slot,
            player_id: slot.player_id,
            x: slot.x,
            y: slot.y,
        }
    }
}

impl From<&LineupSlot> for StoredSlot {
    fn from(slot: &LineupSlot) -> Self {
        Self {
            slot: slot.slot,
            player_id: slot.player_id,
            x: slot.x,
            y: slot.y,
        }
    }
}

fn to_lineup((formation, Json(slots), share_code, updated_at): LineupRow) -> Lineup {
    Lineup {
        formation,
        slots: slots.into_iter().map(Into::into).collect(),
        share_code,
        updated_at,
    }
}

type DraftRow = (String, Json<Vec<StoredSlot>>, chrono::DateTime<chrono::Utc>);

fn to_draft((formation, Json(slots), updated_at): DraftRow) -> LineupDraft {
    LineupDraft {
        formation,
        slots: slots.into_iter().map(Into::into).collect(),
        updated_at,
    }
}

const SAVE_DRAFT: &str = "INSERT INTO f_i_lineup_drafts (user_id, team_id, formation, slots)
    VALUES ($1, $2, $3, $4)
    ON CONFLICT (user_id, team_id) DO UPDATE
       SET formation = EXCLUDED.formation, slots = EXCLUDED.slots, updated_at = NOW()
    RETURNING formation, slots, updated_at";

#[async_trait]
impl LineupRepository for PostgresLineupRepository {
    async fn find_draft(
        &self,
        user_id: Uuid,
        team_id: i64,
    ) -> Result<Option<LineupDraft>, StoreError> {
        sqlx::query_as::<_, DraftRow>("SELECT formation, slots, updated_at FROM f_i_lineup_drafts WHERE user_id = $1 AND team_id = $2")
            .bind(user_id).bind(team_id).fetch_optional(&self.pool).await
            .map(|row| row.map(to_draft)).map_err(store_error)
    }

    async fn save_draft(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: &[LineupSlot],
    ) -> Result<LineupDraft, StoreError> {
        sqlx::query_as::<_, DraftRow>(SAVE_DRAFT)
            .bind(user_id)
            .bind(team_id)
            .bind(formation)
            .bind(Json(slots.iter().map(StoredSlot::from).collect::<Vec<_>>()))
            .fetch_one(&self.pool)
            .await
            .map(to_draft)
            .map_err(store_error)
    }

    async fn publish(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: &[LineupSlot],
        share_code: &str,
    ) -> Result<PublishedLineup, StoreError> {
        let mut tx = self.pool.begin().await.map_err(store_error)?;
        let stored_slots = Json(slots.iter().map(StoredSlot::from).collect::<Vec<_>>());
        let lineup = sqlx::query_as::<_, LineupRow>(
            "INSERT INTO f_i_lineups (id, user_id, team_id, formation, slots, share_code)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (user_id, team_id) DO UPDATE
                SET formation = EXCLUDED.formation, slots = EXCLUDED.slots, updated_at = NOW()
             RETURNING formation, slots, share_code, updated_at",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(team_id)
        .bind(formation)
        .bind(&stored_slots)
        .bind(share_code)
        .fetch_one(&mut *tx)
        .await
        .map(to_lineup)
        .map_err(store_error)?;
        let draft = sqlx::query_as::<_, DraftRow>(SAVE_DRAFT)
            .bind(user_id)
            .bind(team_id)
            .bind(formation)
            .bind(&stored_slots)
            .fetch_one(&mut *tx)
            .await
            .map(to_draft)
            .map_err(store_error)?;
        tx.commit().await.map_err(store_error)?;
        Ok(PublishedLineup { lineup, draft })
    }
    async fn find(&self, user_id: Uuid, team_id: i64) -> Result<Option<Lineup>, StoreError> {
        sqlx::query_as::<_, LineupRow>(
            "SELECT formation, slots, share_code, updated_at FROM f_i_lineups WHERE user_id = $1 AND team_id = $2",
        )
        .bind(user_id)
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(to_lineup))
        .map_err(store_error)
    }

    async fn known_players(&self, player_ids: &[i64]) -> Result<Vec<i64>, StoreError> {
        sqlx::query_scalar::<_, i64>(
            "SELECT DISTINCT player_id FROM f_i_squad_players WHERE player_id = ANY($1)",
        )
        .bind(player_ids)
        .fetch_all(&self.pool)
        .await
        .map_err(store_error)
    }

    async fn upsert(
        &self,
        user_id: Uuid,
        team_id: i64,
        formation: &str,
        slots: &[LineupSlot],
        share_code: &str,
    ) -> Result<Lineup, StoreError> {
        self.publish(user_id, team_id, formation, slots, share_code)
            .await
            .map(|result| result.lineup)
    }
}
