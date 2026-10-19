use crate::lineup::{
    domain::{Lineup, LineupReadError, LineupSlot, Position, SquadPlayer, SquadTeam},
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
                        }
                    },
                )
                .collect()
        })
        .map_err(read_error)
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

#[async_trait]
impl LineupRepository for PostgresLineupRepository {
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

    async fn squad_members(
        &self,
        team_id: i64,
        player_ids: &[i64],
    ) -> Result<Vec<i64>, StoreError> {
        sqlx::query_scalar::<_, i64>(
            "SELECT player_id FROM f_i_squad_players WHERE team_id = $1 AND player_id = ANY($2)",
        )
        .bind(team_id)
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
        sqlx::query_as::<_, LineupRow>(
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
        .bind(Json(slots.iter().map(StoredSlot::from).collect::<Vec<_>>()))
        .bind(share_code)
        .fetch_one(&self.pool)
        .await
        .map(to_lineup)
        .map_err(store_error)
    }
}
