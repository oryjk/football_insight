-- Public lineups remain immutable during private editing. Only explicit publishing updates them.
CREATE TABLE IF NOT EXISTS f_i_lineup_drafts (
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    team_id BIGINT NOT NULL,
    formation TEXT NOT NULL,
    slots JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, team_id)
);

-- Preserve legacy edits (including incomplete lineups) as the initial private draft.
INSERT INTO f_i_lineup_drafts (user_id, team_id, formation, slots, created_at, updated_at)
SELECT user_id, team_id, formation, slots, created_at, updated_at FROM f_i_lineups
ON CONFLICT (user_id, team_id) DO NOTHING;
