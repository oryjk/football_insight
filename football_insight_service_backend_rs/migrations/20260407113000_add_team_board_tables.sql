CREATE TABLE IF NOT EXISTS f_i_team_insight_posts (
    id UUID PRIMARY KEY,
    team_id BIGINT NOT NULL REFERENCES f_i_teams(team_id) ON DELETE CASCADE,
    author_user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    insight_kind VARCHAR(32) NOT NULL,
    title VARCHAR(120) NOT NULL,
    commentary TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_team_insight_posts_team_created
    ON f_i_team_insight_posts (team_id, created_at DESC);

CREATE TABLE IF NOT EXISTS f_i_team_insight_post_snapshots (
    post_id UUID PRIMARY KEY REFERENCES f_i_team_insight_posts(id) ON DELETE CASCADE,
    snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS f_i_team_insight_comments (
    id UUID PRIMARY KEY,
    post_id UUID NOT NULL REFERENCES f_i_team_insight_posts(id) ON DELETE CASCADE,
    author_user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_team_insight_comments_post_created
    ON f_i_team_insight_comments (post_id, created_at ASC);

CREATE TABLE IF NOT EXISTS f_i_team_insight_post_likes (
    post_id UUID NOT NULL REFERENCES f_i_team_insight_posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_team_insight_post_likes_user_id
    ON f_i_team_insight_post_likes (user_id);
