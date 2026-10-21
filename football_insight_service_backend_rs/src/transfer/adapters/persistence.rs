use crate::transfer::{
    domain::{
        CandidatePlayer, CardPlayer, NominationCard, TransferError, TransferIntent, TransferTeam,
        TransferWindow, VoteTally, VoteValue, is_departed, is_returning,
    },
    ports::{
        CardContext, NominateCommand, NominateOutcome, TransferCandidateDirectory,
        TransferRepository, TransferUserAccess, UserProfile,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

pub struct PostgresTransferRepository {
    pool: PgPool,
}

impl PostgresTransferRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn unavailable(error: sqlx::Error) -> TransferError {
    if let Some(db) = error.as_database_error()
        && db.code().as_deref() == Some("23505")
        && db.constraint() == Some("uq_f_i_transfer_nominations_share_code")
    {
        return TransferError::ShareCodeTaken;
    }
    tracing::warn!(error = %error, "transfer store unavailable");
    TransferError::Unavailable
}

/// $1 浏览者（可空）$2 期 $3 球队 $4 意向。只统计正常账号的票；只给有票的卡排名。
const RANKED: &str = "WITH tally AS (
    SELECT n.id, n.team_id, n.player_id, n.intent, n.share_code, n.created_at,
        COUNT(v.user_id) FILTER (WHERE v.value = 1) AS up,
        COUNT(v.user_id) FILTER (WHERE v.value = -1) AS down,
        COALESCE(MAX(v.value) FILTER (WHERE v.user_id = $1), 0)::SMALLINT AS my_vote
    FROM f_i_transfer_nominations n
    LEFT JOIN (f_i_transfer_votes v JOIN f_i_users vu ON vu.id = v.user_id AND vu.status = 'active')
        ON v.nomination_id = n.id
    WHERE n.window_id = $2 AND n.team_id = $3 AND n.intent = $4
    GROUP BY n.id
), ranked AS (
    SELECT t.*, CASE WHEN t.up + t.down > 0 THEN ROW_NUMBER() OVER (
        PARTITION BY t.up + t.down > 0
        ORDER BY t.up - t.down DESC, t.up DESC, t.created_at, t.share_code) END AS rank
    FROM tally t
)";

/// 主教练卡（player_id 为 f_i_team_coaches.coach_id）与球员卡共用：名字、头像、现所在队都回落到主教练表。
const CARD_SELECT: &str = "SELECT r.share_code, r.intent, r.player_id, r.up, r.down, r.my_vote, r.rank, r.created_at,
    COALESCE(p.player_name, c.coach_name) AS player_name,
    COALESCE(p.avatar_storage_url, c.avatar_storage_url) AS avatar_url, p.position, p.shirt_number,
    COALESCE(ct.team_id, cteam.team_id) AS current_team_id,
    COALESCE(ct.team_name, cteam.team_name) AS current_team_name,
    CASE WHEN ct.team_id IS NOT NULL THEN ct.avatar_storage_url ELSE cteam.avatar_storage_url END AS current_team_avatar,
    EXISTS(SELECT 1 FROM f_i_squad_players f WHERE f.player_id = r.player_id AND f.team_id = r.team_id AND NOT f.active) AS former_of_team,
    (EXISTS(SELECT 1 FROM f_i_squad_players a WHERE a.player_id = r.player_id AND a.team_id = r.team_id AND a.active)
        OR COALESCE(c.active AND c.team_id = r.team_id, FALSE)) AS active_in_team
FROM ranked r
LEFT JOIN LATERAL (SELECT sp.player_name, sp.avatar_storage_url, sp.position, sp.shirt_number
    FROM f_i_squad_players sp WHERE sp.player_id = r.player_id
    ORDER BY sp.active DESC, sp.last_season DESC LIMIT 1) p ON TRUE
LEFT JOIN LATERAL (SELECT t.team_id, t.team_name, t.avatar_storage_url
    FROM f_i_squad_players sp JOIN f_i_teams t ON t.team_id = sp.team_id
    WHERE sp.player_id = r.player_id AND sp.active LIMIT 1) ct ON TRUE
LEFT JOIN f_i_team_coaches c ON c.coach_id = r.player_id
LEFT JOIN f_i_teams cteam ON cteam.team_id = c.team_id AND c.active";

#[derive(FromRow)]
struct CardRow {
    share_code: String,
    intent: String,
    player_id: i64,
    up: i64,
    down: i64,
    my_vote: i16,
    rank: Option<i64>,
    created_at: DateTime<Utc>,
    player_name: Option<String>,
    avatar_url: Option<String>,
    position: Option<String>,
    shirt_number: Option<i32>,
    current_team_id: Option<i64>,
    current_team_name: Option<String>,
    current_team_avatar: Option<String>,
    former_of_team: bool,
    active_in_team: bool,
}

impl CardRow {
    fn into_card(self) -> Result<NominationCard, TransferError> {
        let intent = TransferIntent::parse(&self.intent).ok_or(TransferError::Unavailable)?;
        let current_team = match (self.current_team_id, self.current_team_name) {
            (Some(team_id), Some(team_name)) => Some(TransferTeam {
                team_id,
                team_name,
                avatar_url: self.current_team_avatar,
            }),
            _ => None,
        };
        Ok(NominationCard {
            share_code: self.share_code,
            intent,
            returning: is_returning(intent, self.former_of_team),
            player: CardPlayer {
                player_id: self.player_id,
                name: self.player_name.unwrap_or_else(|| "未知球员".into()),
                avatar_url: self.avatar_url,
                position: self.position,
                shirt_number: self.shirt_number,
                current_team,
                departed: is_departed(intent, self.active_in_team),
            },
            up: self.up,
            down: self.down,
            my_vote: self.my_vote,
            rank: self.rank,
            created_at: self.created_at,
        })
    }
}

#[derive(FromRow)]
struct WindowRow {
    id: i64,
    code: String,
    name: String,
    opens_at: DateTime<Utc>,
    closes_at: DateTime<Utc>,
}

impl From<WindowRow> for TransferWindow {
    fn from(row: WindowRow) -> Self {
        Self {
            id: row.id,
            code: row.code,
            name: row.name,
            opens_at: row.opens_at,
            closes_at: row.closes_at,
        }
    }
}

const WINDOW_COLUMNS: &str = "id, code, name, opens_at, closes_at";

async fn lock_active_user(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    exclusive: bool,
) -> Result<(), TransferError> {
    let lock = if exclusive { "FOR UPDATE" } else { "FOR SHARE" };
    sqlx::query_scalar::<_, Uuid>(&format!(
        "SELECT id FROM f_i_users WHERE id = $1 AND status = 'active' {lock}"
    ))
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(unavailable)?
    .map(|_| ())
    .ok_or(TransferError::Unauthorized)
}

async fn tally(
    tx: &mut Transaction<'_, Postgres>,
    nomination_id: Uuid,
    user_id: Uuid,
) -> Result<VoteTally, TransferError> {
    let (up, down, my_vote) = sqlx::query_as::<_, (i64, i64, i16)>(
        "SELECT COUNT(*) FILTER (WHERE v.value = 1), COUNT(*) FILTER (WHERE v.value = -1),
            COALESCE(MAX(v.value) FILTER (WHERE v.user_id = $2), 0)::SMALLINT
         FROM f_i_transfer_votes v JOIN f_i_users u ON u.id = v.user_id AND u.status = 'active'
         WHERE v.nomination_id = $1",
    )
    .bind(nomination_id)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(unavailable)?;
    Ok(VoteTally { up, down, my_vote })
}

#[async_trait]
impl TransferRepository for PostgresTransferRepository {
    async fn current_window(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Option<TransferWindow>, TransferError> {
        let open = sqlx::query_as::<_, WindowRow>(&format!(
            "SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows WHERE opens_at <= $1 AND $1 < closes_at ORDER BY opens_at DESC LIMIT 1"
        ))
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;
        if let Some(window) = open {
            return Ok(Some(window.into()));
        }
        sqlx::query_as::<_, WindowRow>(&format!(
            "SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows WHERE closes_at <= $1 ORDER BY closes_at DESC LIMIT 1"
        ))
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(unavailable)
    }

    async fn window_by_code(&self, code: &str) -> Result<Option<TransferWindow>, TransferError> {
        sqlx::query_as::<_, WindowRow>(&format!(
            "SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows WHERE code = $1"
        ))
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Into::into))
        .map_err(unavailable)
    }

    async fn board(
        &self,
        window_id: i64,
        team_id: i64,
        intent: TransferIntent,
        offset: i64,
        limit: i64,
        viewer: Option<Uuid>,
    ) -> Result<(Vec<NominationCard>, i64), TransferError> {
        let rows = sqlx::query_as::<_, CardRow>(&format!(
            "{RANKED} {CARD_SELECT} WHERE r.rank IS NOT NULL ORDER BY r.rank LIMIT $5 OFFSET $6"
        ))
        .bind(viewer)
        .bind(window_id)
        .bind(team_id)
        .bind(intent.as_str())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(unavailable)?;
        let total: i64 = sqlx::query_scalar(&format!(
            "{RANKED} SELECT COUNT(*) FROM ranked WHERE rank IS NOT NULL"
        ))
        .bind(viewer)
        .bind(window_id)
        .bind(team_id)
        .bind(intent.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(unavailable)?;
        let items = rows
            .into_iter()
            .map(CardRow::into_card)
            .collect::<Result<_, _>>()?;
        Ok((items, total))
    }

    async fn card(
        &self,
        share_code: &str,
        viewer: Option<Uuid>,
    ) -> Result<Option<CardContext>, TransferError> {
        let head = sqlx::query_as::<_, (i64, i64, String, String, Option<String>)>(
            "SELECT n.window_id, n.team_id, n.intent, t.team_name, t.avatar_storage_url
             FROM f_i_transfer_nominations n JOIN f_i_teams t ON t.team_id = n.team_id WHERE n.share_code = $1",
        )
        .bind(share_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(unavailable)?;
        let Some((window_id, team_id, intent, team_name, team_avatar)) = head else {
            return Ok(None);
        };
        let window: TransferWindow = sqlx::query_as::<_, WindowRow>(&format!(
            "SELECT {WINDOW_COLUMNS} FROM f_i_transfer_windows WHERE id = $1"
        ))
        .bind(window_id)
        .fetch_one(&self.pool)
        .await
        .map_err(unavailable)?
        .into();
        let row = sqlx::query_as::<_, CardRow>(&format!(
            "{RANKED} {CARD_SELECT} WHERE r.share_code = $5"
        ))
        .bind(viewer)
        .bind(window_id)
        .bind(team_id)
        .bind(&intent)
        .bind(share_code)
        .fetch_one(&self.pool)
        .await
        .map_err(unavailable)?;
        Ok(Some(CardContext {
            card: row.into_card()?,
            team: TransferTeam {
                team_id,
                team_name,
                avatar_url: team_avatar,
            },
            window,
        }))
    }

    async fn nominate(&self, command: NominateCommand) -> Result<NominateOutcome, TransferError> {
        let mut tx = self.pool.begin().await.map_err(unavailable)?;
        // 锁提名人：同一人并发提名时串行，名额计数不会被绕过。
        lock_active_user(&mut tx, command.user_id, true).await?;
        let find = "SELECT id, share_code FROM f_i_transfer_nominations
            WHERE window_id = $1 AND team_id = $2 AND player_id = $3 AND intent = $4";
        let existing = sqlx::query_as::<_, (Uuid, String)>(find)
            .bind(command.window_id)
            .bind(command.team_id)
            .bind(command.player_id)
            .bind(command.intent.as_str())
            .fetch_optional(&mut *tx)
            .await
            .map_err(unavailable)?;
        let (id, share_code, created) = match existing {
            Some((id, code)) => (id, code, false),
            None => {
                let created_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM f_i_transfer_nominations WHERE window_id = $1 AND created_by = $2 AND intent = $3",
                )
                .bind(command.window_id).bind(command.user_id).bind(command.intent.as_str())
                .fetch_one(&mut *tx).await.map_err(unavailable)?;
                // 上限规则在 domain（NOMINATION_LIMIT），由 application 传入；这里只服从传入值。
                if created_count >= command.limit {
                    return Err(TransferError::NominationLimit);
                }
                let inserted = sqlx::query_as::<_, (Uuid, String)>(
                    "INSERT INTO f_i_transfer_nominations (window_id, team_id, player_id, intent, share_code, created_by)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     ON CONFLICT (window_id, team_id, player_id, intent) DO NOTHING RETURNING id, share_code",
                )
                .bind(command.window_id).bind(command.team_id).bind(command.player_id).bind(command.intent.as_str())
                .bind(&command.share_code).bind(command.user_id)
                .fetch_optional(&mut *tx).await.map_err(unavailable)?;
                match inserted {
                    Some((id, code)) => (id, code, true),
                    // 另一位球迷同时提名了同一张卡：沿用对方的卡，给它加票。
                    None => {
                        let (id, code) = sqlx::query_as::<_, (Uuid, String)>(find)
                            .bind(command.window_id)
                            .bind(command.team_id)
                            .bind(command.player_id)
                            .bind(command.intent.as_str())
                            .fetch_one(&mut *tx)
                            .await
                            .map_err(unavailable)?;
                        (id, code, false)
                    }
                }
            }
        };
        sqlx::query(
            "INSERT INTO f_i_transfer_votes (nomination_id, user_id, value) VALUES ($1, $2, 1)
             ON CONFLICT (nomination_id, user_id) DO UPDATE SET value = 1, updated_at = NOW()",
        )
        .bind(id)
        .bind(command.user_id)
        .execute(&mut *tx)
        .await
        .map_err(unavailable)?;
        tx.commit().await.map_err(unavailable)?;
        Ok(NominateOutcome {
            share_code,
            created,
        })
    }

    async fn set_vote(
        &self,
        share_code: &str,
        user_id: Uuid,
        value: Option<VoteValue>,
    ) -> Result<VoteTally, TransferError> {
        let mut tx = self.pool.begin().await.map_err(unavailable)?;
        lock_active_user(&mut tx, user_id, false).await?;
        let id: Uuid =
            sqlx::query_scalar("SELECT id FROM f_i_transfer_nominations WHERE share_code = $1")
                .bind(share_code)
                .fetch_optional(&mut *tx)
                .await
                .map_err(unavailable)?
                .ok_or(TransferError::NotFound)?;
        match value {
            Some(value) => sqlx::query(
                "INSERT INTO f_i_transfer_votes (nomination_id, user_id, value) VALUES ($1, $2, $3)
                 ON CONFLICT (nomination_id, user_id) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()",
            )
            .bind(id).bind(user_id).bind(value.as_i16()),
            None => sqlx::query("DELETE FROM f_i_transfer_votes WHERE nomination_id = $1 AND user_id = $2")
                .bind(id).bind(user_id),
        }
        .execute(&mut *tx).await.map_err(unavailable)?;
        let result = tally(&mut tx, id, user_id).await?;
        tx.commit().await.map_err(unavailable)?;
        Ok(result)
    }
}

const CANDIDATE_COLUMNS: &str =
    "sp.player_id, sp.player_name, sp.shirt_number, sp.position, sp.avatar_storage_url";
const POSITION_ORDER: &str = "CASE sp.position WHEN 'GK' THEN 0 WHEN 'DF' THEN 1 WHEN 'MF' THEN 2 WHEN 'FW' THEN 3 ELSE 4 END";

type CandidateRow = (
    i64,
    String,
    Option<i32>,
    Option<String>,
    Option<String>,
    Option<i64>,
    Option<String>,
    Option<String>,
);

fn candidate(row: CandidateRow) -> CandidatePlayer {
    let (player_id, name, shirt_number, position, avatar_url, team_id, team_name, team_avatar) =
        row;
    CandidatePlayer {
        player_id,
        name,
        shirt_number,
        position,
        avatar_url,
        current_team: match (team_id, team_name) {
            (Some(team_id), Some(team_name)) => Some(TransferTeam {
                team_id,
                team_name,
                avatar_url: team_avatar,
            }),
            _ => None,
        },
    }
}

#[async_trait]
impl TransferCandidateDirectory for PostgresTransferRepository {
    async fn find_team(&self, team_id: i64) -> Result<Option<TransferTeam>, TransferError> {
        sqlx::query_as::<_, (i64, String, Option<String>)>(
            "SELECT team_id, team_name, avatar_storage_url FROM f_i_teams WHERE team_id = $1",
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|(team_id, team_name, avatar_url)| TransferTeam {
                team_id,
                team_name,
                avatar_url,
            })
        })
        .map_err(unavailable)
    }

    async fn player_rows(&self, player_id: i64) -> Result<Vec<(i64, bool)>, TransferError> {
        sqlx::query_as("SELECT team_id, active FROM f_i_squad_players WHERE player_id = $1")
            .bind(player_id)
            .fetch_all(&self.pool)
            .await
            .map_err(unavailable)
    }

    async fn coach_rows(&self, coach_id: i64) -> Result<Vec<(i64, bool)>, TransferError> {
        sqlx::query_as("SELECT team_id, active FROM f_i_team_coaches WHERE coach_id = $1")
            .bind(coach_id)
            .fetch_all(&self.pool)
            .await
            .map_err(unavailable)
    }

    async fn team_coach(&self, team_id: i64) -> Result<Option<CandidatePlayer>, TransferError> {
        // 主教练没有号码和位置；前端只显示姓名与球队。
        sqlx::query_as::<_, CandidateRow>(
            "SELECT c.coach_id, c.coach_name, NULL::INT, NULL::TEXT, c.avatar_storage_url,
                    t.team_id, t.team_name, t.avatar_storage_url
             FROM f_i_team_coaches c JOIN f_i_teams t ON t.team_id = c.team_id
             WHERE c.team_id = $1 AND c.active",
        )
        .bind(team_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(candidate))
        .map_err(unavailable)
    }

    async fn team_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
        sqlx::query_as::<_, CandidateRow>(&format!(
            "SELECT {CANDIDATE_COLUMNS}, t.team_id, t.team_name, t.avatar_storage_url
             FROM f_i_squad_players sp JOIN f_i_teams t ON t.team_id = sp.team_id
             WHERE sp.team_id = $1 AND sp.active
             ORDER BY {POSITION_ORDER}, sp.shirt_number NULLS LAST, sp.player_id"
        ))
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(candidate).collect())
        .map_err(unavailable)
    }

    async fn former_players(&self, team_id: i64) -> Result<Vec<CandidatePlayer>, TransferError> {
        sqlx::query_as::<_, CandidateRow>(&format!(
            "SELECT {CANDIDATE_COLUMNS}, ct.team_id, ct.team_name, ct.avatar_storage_url
             FROM f_i_squad_players sp
             LEFT JOIN LATERAL (SELECT t.team_id, t.team_name, t.avatar_storage_url
                 FROM f_i_squad_players a JOIN f_i_teams t ON t.team_id = a.team_id
                 WHERE a.player_id = sp.player_id AND a.active LIMIT 1) ct ON TRUE
             WHERE sp.team_id = $1 AND NOT sp.active
             ORDER BY sp.last_season DESC, {POSITION_ORDER}, sp.player_id"
        ))
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(candidate).collect())
        .map_err(unavailable)
    }

    async fn other_active_players(
        &self,
        team_id: i64,
    ) -> Result<Vec<CandidatePlayer>, TransferError> {
        sqlx::query_as::<_, CandidateRow>(&format!(
            "SELECT {CANDIDATE_COLUMNS}, t.team_id, t.team_name, t.avatar_storage_url
             FROM f_i_squad_players sp JOIN f_i_teams t ON t.team_id = sp.team_id
             WHERE sp.active AND sp.team_id <> $1
               AND NOT EXISTS (SELECT 1 FROM f_i_squad_players f WHERE f.player_id = sp.player_id AND f.team_id = $1)
             ORDER BY t.team_name, {POSITION_ORDER}, sp.shirt_number NULLS LAST, sp.player_id"
        ))
        .bind(team_id)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(candidate).collect())
        .map_err(unavailable)
    }
}

#[async_trait]
impl TransferUserAccess for PostgresTransferRepository {
    async fn profile(&self, user_id: Uuid) -> Result<Option<UserProfile>, TransferError> {
        sqlx::query_as::<_, (String, Option<i64>)>(
            "SELECT status, favorite_team_id FROM f_i_users WHERE id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map(|row| {
            row.map(|(status, favorite_team_id)| UserProfile {
                active: status == "active",
                favorite_team_id,
            })
        })
        .map_err(unavailable)
    }
}
