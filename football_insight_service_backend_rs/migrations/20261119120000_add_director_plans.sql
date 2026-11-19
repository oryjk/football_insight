-- 「我来当总监」休赛期建队选人功能表结构

-- 1. 精选引援候选池（重点覆盖 K 联赛、J 联赛、绯闻外援及国内重点目标）
CREATE TABLE IF NOT EXISTS f_i_director_candidates (
    id BIGSERIAL PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    player_name TEXT NOT NULL,
    position TEXT NOT NULL CHECK (position IN ('GK', 'DF', 'MF', 'FW')),
    country TEXT NOT NULL,
    is_foreign BOOLEAN NOT NULL DEFAULT TRUE,
    league_tag TEXT NOT NULL,
    current_club TEXT NOT NULL,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    target_team_id BIGINT,
    avatar_url TEXT,
    sort_order INTEGER NOT NULL DEFAULT 100,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_director_candidates_filter
    ON f_i_director_candidates (active, is_foreign, league_tag, sort_order);

-- 2. 用户建队方案（同窗口 + 同球队 + 同用户唯一）
CREATE TABLE IF NOT EXISTS f_i_director_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    window_id BIGINT NOT NULL REFERENCES f_i_transfer_windows(id) ON DELETE CASCADE,
    team_id BIGINT NOT NULL REFERENCES f_i_teams(team_id),
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    share_code TEXT NOT NULL UNIQUE,
    circle_code TEXT NOT NULL,
    parent_share_code TEXT,
    foreign_stays JSONB NOT NULL DEFAULT '[]'::jsonb,
    foreign_leaves JSONB NOT NULL DEFAULT '[]'::jsonb,
    foreign_ins JSONB NOT NULL DEFAULT '[]'::jsonb,
    domestic_ins JSONB NOT NULL DEFAULT '[]'::jsonb,
    domestic_outs JSONB NOT NULL DEFAULT '[]'::jsonb,
    style_tag TEXT NOT NULL,
    manifesto TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_f_i_director_plans_window_team_user UNIQUE (window_id, team_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_director_plans_circle
    ON f_i_director_plans (window_id, team_id, circle_code, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_f_i_director_plans_team
    ON f_i_director_plans (window_id, team_id, updated_at DESC);

-- 3. 方案赞/踩（互斥，value IN (1, -1)）
CREATE TABLE IF NOT EXISTS f_i_director_plan_reactions (
    plan_id UUID NOT NULL REFERENCES f_i_director_plans(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    value SMALLINT NOT NULL CHECK (value IN (1, -1)),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (plan_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_director_plan_reactions_plan
    ON f_i_director_plan_reactions (plan_id, value);

-- 4. 方案一句话锐评
CREATE TABLE IF NOT EXISTS f_i_director_plan_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES f_i_director_plans(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_f_i_director_plan_comments_plan
    ON f_i_director_plan_comments (plan_id, created_at DESC);
