use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::team_board::{
    domain::team_board::{
        NewTeamBoardComment, NewTeamBoardPost, TeamBoardComment, TeamBoardLikeSummary,
        TeamBoardPost, TeamBoardPostAuthor, TeamBoardSnapshot,
    },
    ports::team_board_repository::TeamBoardRepository,
};

#[derive(Clone)]
pub struct PostgresTeamBoardRepository {
    pool: PgPool,
}

impl PostgresTeamBoardRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamBoardRepository for PostgresTeamBoardRepository {
    async fn list_posts(
        &self,
        team_id: i64,
        viewer_user_id: Uuid,
    ) -> anyhow::Result<Vec<TeamBoardPost>> {
        let post_rows = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT
                p.id AS post_id,
                p.team_id,
                p.insight_kind,
                p.title,
                p.commentary,
                p.created_at,
                u.id AS author_user_id,
                COALESCE(u.display_name, u.account_identifier, '匿名用户') AS author_display_name,
                u.avatar_url AS author_avatar_url,
                s.snapshot AS snapshot,
                COALESCE(l.like_count, 0) AS like_count,
                COALESCE(c.comment_count, 0) AS comment_count,
                CASE WHEN vl.post_id IS NULL THEN FALSE ELSE TRUE END AS liked_by_viewer
              FROM f_i_team_insight_posts p
              JOIN f_i_users u ON u.id = p.author_user_id
              JOIN f_i_team_insight_post_snapshots s ON s.post_id = p.id
              LEFT JOIN (
                  SELECT post_id, COUNT(*)::bigint AS like_count
                    FROM f_i_team_insight_post_likes
                   GROUP BY post_id
              ) l ON l.post_id = p.id
              LEFT JOIN (
                  SELECT post_id, COUNT(*)::bigint AS comment_count
                    FROM f_i_team_insight_comments
                   GROUP BY post_id
              ) c ON c.post_id = p.id
              LEFT JOIN (
                  SELECT post_id
                    FROM f_i_team_insight_post_likes
                   WHERE user_id = $2
              ) vl ON vl.post_id = p.id
             WHERE p.team_id = $1
             ORDER BY p.created_at DESC, p.id DESC
            "#,
        )
        .bind(team_id)
        .bind(viewer_user_id)
        .fetch_all(&self.pool)
        .await?;

        let post_ids = post_rows.iter().map(|row| row.post_id).collect::<Vec<_>>();
        let mut comments_by_post_id: HashMap<Uuid, Vec<TeamBoardComment>> = HashMap::new();

        if !post_ids.is_empty() {
            let comment_rows = sqlx::query_as::<_, CommentRow>(
                r#"
                SELECT
                    c.id AS comment_id,
                    c.post_id,
                    c.content,
                    c.created_at,
                    u.id AS author_user_id,
                    COALESCE(u.display_name, u.account_identifier, '匿名用户') AS author_display_name,
                    u.avatar_url AS author_avatar_url
                  FROM f_i_team_insight_comments c
                  JOIN f_i_users u ON u.id = c.author_user_id
                 WHERE c.post_id = ANY($1)
                 ORDER BY c.created_at ASC, c.id ASC
                "#,
            )
            .bind(&post_ids)
            .fetch_all(&self.pool)
            .await?;

            for row in comment_rows {
                comments_by_post_id
                    .entry(row.post_id)
                    .or_default()
                    .push(row.into());
            }
        }

        Ok(post_rows
            .into_iter()
            .map(|row| {
                let comments = comments_by_post_id.remove(&row.post_id).unwrap_or_default();
                row.into_post(comments)
            })
            .collect())
    }

    async fn create_post(&self, input: NewTeamBoardPost) -> anyhow::Result<TeamBoardPost> {
        let post_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO f_i_team_insight_posts (
                id,
                team_id,
                author_user_id,
                insight_kind,
                title,
                commentary,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            "#,
        )
        .bind(post_id)
        .bind(input.team_id)
        .bind(input.author_user_id)
        .bind(input.insight_kind)
        .bind(&input.title)
        .bind(&input.commentary)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO f_i_team_insight_post_snapshots (post_id, snapshot, created_at)
            VALUES ($1, $2, NOW())
            "#,
        )
        .bind(post_id)
        .bind(Json(&input.snapshot))
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT
                p.id AS post_id,
                p.team_id,
                p.insight_kind,
                p.title,
                p.commentary,
                p.created_at,
                u.id AS author_user_id,
                COALESCE(u.display_name, u.account_identifier, '匿名用户') AS author_display_name,
                u.avatar_url AS author_avatar_url,
                s.snapshot AS snapshot,
                0::bigint AS like_count,
                0::bigint AS comment_count,
                FALSE AS liked_by_viewer
              FROM f_i_team_insight_posts p
              JOIN f_i_users u ON u.id = p.author_user_id
              JOIN f_i_team_insight_post_snapshots s ON s.post_id = p.id
             WHERE p.id = $1
            "#,
        )
        .bind(post_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into_post(Vec::new()))
    }

    async fn add_comment(&self, input: NewTeamBoardComment) -> anyhow::Result<TeamBoardComment> {
        let comment_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO f_i_team_insight_comments (id, post_id, author_user_id, content, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
        )
        .bind(comment_id)
        .bind(input.post_id)
        .bind(input.author_user_id)
        .bind(&input.content)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, CommentRow>(
            r#"
            SELECT
                c.id AS comment_id,
                c.post_id,
                c.content,
                c.created_at,
                u.id AS author_user_id,
                COALESCE(u.display_name, u.account_identifier, '匿名用户') AS author_display_name,
                u.avatar_url AS author_avatar_url
              FROM f_i_team_insight_comments c
              JOIN f_i_users u ON u.id = c.author_user_id
             WHERE c.id = $1
            "#,
        )
        .bind(comment_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn toggle_like(
        &self,
        post_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<TeamBoardLikeSummary> {
        let deleted = sqlx::query(
            r#"
            DELETE FROM f_i_team_insight_post_likes
             WHERE post_id = $1
               AND user_id = $2
            "#,
        )
        .bind(post_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        let liked_by_viewer = if deleted.rows_affected() > 0 {
            false
        } else {
            sqlx::query(
                r#"
                INSERT INTO f_i_team_insight_post_likes (post_id, user_id, created_at)
                VALUES ($1, $2, NOW())
                ON CONFLICT (post_id, user_id) DO NOTHING
                "#,
            )
            .bind(post_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
            true
        };

        let like_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)::bigint
              FROM f_i_team_insight_post_likes
             WHERE post_id = $1
            "#,
        )
        .bind(post_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(TeamBoardLikeSummary {
            post_id,
            liked_by_viewer,
            like_count,
        })
    }
}

#[derive(sqlx::FromRow)]
struct PostRow {
    post_id: Uuid,
    team_id: i64,
    insight_kind: crate::team_board::domain::team_board::TeamBoardInsightKind,
    title: String,
    commentary: String,
    created_at: chrono::DateTime<Utc>,
    author_user_id: Uuid,
    author_display_name: String,
    author_avatar_url: Option<String>,
    snapshot: Json<TeamBoardSnapshot>,
    like_count: i64,
    comment_count: i64,
    liked_by_viewer: bool,
}

impl PostRow {
    fn into_post(self, comments: Vec<TeamBoardComment>) -> TeamBoardPost {
        TeamBoardPost {
            post_id: self.post_id,
            team_id: self.team_id,
            insight_kind: self.insight_kind,
            insight_label: self.insight_kind.label().to_string(),
            title: self.title,
            commentary: self.commentary,
            author: TeamBoardPostAuthor {
                user_id: self.author_user_id,
                display_name: self.author_display_name,
                avatar_url: self.author_avatar_url,
            },
            snapshot: self.snapshot.0,
            like_count: self.like_count,
            comment_count: self.comment_count,
            liked_by_viewer: self.liked_by_viewer,
            created_at: self.created_at,
            comments,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CommentRow {
    comment_id: Uuid,
    post_id: Uuid,
    content: String,
    created_at: chrono::DateTime<Utc>,
    author_user_id: Uuid,
    author_display_name: String,
    author_avatar_url: Option<String>,
}

impl From<CommentRow> for TeamBoardComment {
    fn from(value: CommentRow) -> Self {
        Self {
            comment_id: value.comment_id,
            post_id: value.post_id,
            author: TeamBoardPostAuthor {
                user_id: value.author_user_id,
                display_name: value.author_display_name,
                avatar_url: value.author_avatar_url,
            },
            content: value.content,
            created_at: value.created_at,
        }
    }
}
