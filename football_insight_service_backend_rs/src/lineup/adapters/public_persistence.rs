use crate::lineup::{
    domain::{
        LineupAuthor, LineupLikeState, LineupPublicError, LineupReaction, LineupSlot, LineupSort,
        PublicLineupItem, PublicLineupQuery, SharedLineup, SquadTeam,
    },
    ports::PublicLineupRepository,
};
use async_trait::async_trait;
use serde::Deserialize;
use sqlx::{FromRow, PgPool, types::Json};
use uuid::Uuid;

pub struct PostgresPublicLineupRepository {
    pool: PgPool,
}

impl PostgresPublicLineupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// Shared by every public SQL surface. Save validation guarantees unique, valid slots;
// requiring eleven distinct non-null players also rejects malformed legacy records.
const VISIBLE: &str = "u.status = 'active'
    AND jsonb_typeof(l.slots) = 'array' AND jsonb_array_length(l.slots) = 11
    AND (SELECT COUNT(DISTINCT s->>'player_id') FROM jsonb_array_elements(l.slots) s
         WHERE jsonb_typeof(s->'player_id') = 'number') = 11";

const LIKES: &str = "(SELECT COUNT(*) FROM f_i_lineup_likes ll
    JOIN f_i_users lu ON lu.id = ll.user_id AND lu.status = 'active' WHERE ll.lineup_id = l.id)";
const LIKED: &str = "EXISTS(SELECT 1 FROM f_i_lineup_likes me
    JOIN f_i_users mu ON mu.id = me.user_id AND mu.status = 'active'
    WHERE me.lineup_id = l.id AND me.user_id = $2)";
const DISLIKED: &str = "EXISTS(SELECT 1 FROM f_i_lineup_dislikes me
    JOIN f_i_users mu ON mu.id = me.user_id AND mu.status = 'active'
    WHERE me.lineup_id = l.id AND me.user_id = $2)";

#[derive(FromRow)]
struct ItemRow {
    share_code: String,
    formation: String,
    display_name: String,
    avatar_url: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    likes_count: i64,
    liked_by_me: bool,
    disliked_by_me: bool,
}

impl From<ItemRow> for PublicLineupItem {
    fn from(row: ItemRow) -> Self {
        Self {
            share_code: row.share_code,
            formation: row.formation,
            author: LineupAuthor {
                display_name: row.display_name,
                avatar_url: row.avatar_url,
            },
            created_at: row.created_at,
            updated_at: row.updated_at,
            likes: LineupLikeState {
                likes_count: row.likes_count,
                liked_by_me: row.liked_by_me,
                disliked_by_me: row.disliked_by_me,
            },
        }
    }
}

fn columns() -> String {
    format!(
        "l.share_code, l.formation, COALESCE(NULLIF(u.display_name, ''), '球迷') AS display_name,
        u.avatar_url, l.created_at, l.updated_at, {LIKES} AS likes_count, {LIKED} AS liked_by_me, {DISLIKED} AS disliked_by_me"
    )
}

fn unavailable(error: sqlx::Error) -> LineupPublicError {
    tracing::warn!(error = %error, "public lineup persistence unavailable");
    LineupPublicError::Unavailable
}

#[derive(Deserialize)]
struct StoredSlot {
    slot: u8,
    player_id: Option<i64>,
    x: f64,
    y: f64,
}

#[async_trait]
impl PublicLineupRepository for PostgresPublicLineupRepository {
    async fn list(
        &self,
        team_id: i64,
        query: PublicLineupQuery,
        viewer: Option<Uuid>,
    ) -> Result<(Vec<PublicLineupItem>, i64), LineupPublicError> {
        // A consistent snapshot keeps total and items coherent while saves/likes occur.
        let mut tx = self.pool.begin().await.map_err(unavailable)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(unavailable)?;
        let total = sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM f_i_lineups l JOIN f_i_users u ON u.id = l.user_id WHERE l.team_id = $1 AND {VISIBLE}"))
            .bind(team_id).fetch_one(&mut *tx).await.map_err(unavailable)?;
        let order = match query.sort {
            LineupSort::Latest => "l.created_at DESC, l.id DESC",
            LineupSort::Likes => "likes_count DESC, l.created_at DESC, l.id DESC",
        };
        let rows = sqlx::query_as::<_, ItemRow>(&format!("SELECT {} FROM f_i_lineups l JOIN f_i_users u ON u.id = l.user_id WHERE l.team_id = $1 AND {VISIBLE} ORDER BY {order} LIMIT $3 OFFSET $4", columns()))
            .bind(team_id).bind(viewer).bind(i64::from(query.page_size)).bind(query.offset())
            .fetch_all(&mut *tx).await.map_err(unavailable)?;
        tx.commit().await.map_err(unavailable)?;
        Ok((rows.into_iter().map(Into::into).collect(), total))
    }

    async fn shared(
        &self,
        code: &str,
        viewer: Option<Uuid>,
    ) -> Result<Option<SharedLineup>, LineupPublicError> {
        #[derive(FromRow)]
        struct Row {
            #[sqlx(flatten)]
            item: ItemRow,
            team_id: i64,
            team_name: String,
            team_avatar: Option<String>,
            slots: Json<Vec<StoredSlot>>,
        }
        let row = sqlx::query_as::<_, Row>(&format!("SELECT {}, l.team_id, t.team_name, t.avatar_storage_url AS team_avatar, l.slots
            FROM f_i_lineups l JOIN f_i_users u ON u.id = l.user_id JOIN f_i_teams t ON t.team_id = l.team_id
            WHERE l.share_code = $1 AND {VISIBLE}", columns()))
            .bind(code).bind(viewer).fetch_optional(&self.pool).await.map_err(unavailable)?;
        Ok(row.map(|row| SharedLineup {
            item: row.item.into(),
            team: SquadTeam {
                team_id: row.team_id,
                team_name: row.team_name,
                avatar_url: row.team_avatar,
            },
            slots: row
                .slots
                .0
                .into_iter()
                .map(|s| LineupSlot {
                    slot: s.slot,
                    player_id: s.player_id,
                    x: s.x,
                    y: s.y,
                })
                .collect(),
            players: vec![],
        }))
    }

    async fn set_reaction(
        &self,
        code: &str,
        user_id: Uuid,
        reaction: Option<LineupReaction>,
    ) -> Result<LineupLikeState, LineupPublicError> {
        let mut tx = self.pool.begin().await.map_err(unavailable)?;
        // Lock actor state until commit so a concurrent account disable cannot authorize a write.
        let actor = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM f_i_users WHERE id = $1 AND status = 'active' FOR SHARE",
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(unavailable)?;
        if actor.is_none() {
            return Err(LineupPublicError::Unauthorized);
        }
        // Serialize modifications per lineup, including owner saves; lock author status too.
        let id = sqlx::query_scalar::<_, Uuid>(&format!(
            "SELECT l.id FROM f_i_lineups l JOIN f_i_users u ON u.id = l.user_id
            JOIN f_i_teams t ON t.team_id = l.team_id
            WHERE l.share_code = $1 AND {VISIBLE} FOR UPDATE OF l FOR SHARE OF u"
        ))
        .bind(code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(unavailable)?
        .ok_or(LineupPublicError::NotFound)?;
        // Delete the opposite vote (or both for neutral) under the lineup lock.
        if reaction != Some(LineupReaction::Like) {
            sqlx::query("DELETE FROM f_i_lineup_likes WHERE lineup_id = $1 AND user_id = $2")
                .bind(id)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;
        }
        if reaction != Some(LineupReaction::Dislike) {
            sqlx::query("DELETE FROM f_i_lineup_dislikes WHERE lineup_id = $1 AND user_id = $2")
                .bind(id)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;
        }
        if reaction == Some(LineupReaction::Like) {
            sqlx::query("INSERT INTO f_i_lineup_likes(lineup_id,user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING")
                .bind(id).bind(user_id).execute(&mut *tx).await.map_err(unavailable)?;
        } else if reaction == Some(LineupReaction::Dislike) {
            sqlx::query("INSERT INTO f_i_lineup_dislikes(lineup_id,user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING")
                .bind(id)
                .bind(user_id)
                .execute(&mut *tx)
                .await
                .map_err(unavailable)?;
        }
        let (likes_count, liked_by_me, disliked_by_me) = sqlx::query_as::<_, (i64, bool, bool)>(
            &format!("SELECT {LIKES}, {LIKED}, {DISLIKED} FROM f_i_lineups l WHERE l.id = $1"),
        )
        .bind(id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(unavailable)?;
        tx.commit().await.map_err(unavailable)?;
        Ok(LineupLikeState {
            likes_count,
            liked_by_me,
            disliked_by_me,
        })
    }
}
