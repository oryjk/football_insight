-- 用户的"我的最佳阵容"：每人每队一套，保存即覆盖。球员只存 player_id，读取时再关联 f_i_squad_players。
CREATE TABLE IF NOT EXISTS f_i_lineups (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    team_id BIGINT NOT NULL,
    formation TEXT NOT NULL,
    slots JSONB NOT NULL,
    share_code TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_f_i_lineups_user_team UNIQUE (user_id, team_id),
    CONSTRAINT uq_f_i_lineups_share_code UNIQUE (share_code)
);
