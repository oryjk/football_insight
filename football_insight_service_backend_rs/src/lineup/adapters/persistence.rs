use crate::lineup::{
    domain::{LineupReadError, Position, SquadPlayer, SquadTeam},
    ports::SquadRepository,
};
use async_trait::async_trait;
use sqlx::PgPool;

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
