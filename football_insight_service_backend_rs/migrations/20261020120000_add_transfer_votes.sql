-- 转会投票：每期一个窗口；"期 + 球队 + 球员 + 意向"唯一的一张提名卡；每人每卡一票（+1 赞成 / -1 踩）。
CREATE TABLE f_i_transfer_windows (
    id BIGSERIAL PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    opens_at TIMESTAMPTZ NOT NULL,
    closes_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT uq_f_i_transfer_windows_code UNIQUE (code),
    CONSTRAINT ck_f_i_transfer_windows_range CHECK (closes_at > opens_at)
);

CREATE TABLE f_i_transfer_nominations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    window_id BIGINT NOT NULL REFERENCES f_i_transfer_windows(id) ON DELETE CASCADE,
    team_id BIGINT NOT NULL REFERENCES f_i_teams(team_id),
    player_id BIGINT NOT NULL,
    intent TEXT NOT NULL,
    share_code TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_f_i_transfer_nominations_intent CHECK (intent IN ('out', 'stay', 'in')),
    CONSTRAINT uq_f_i_transfer_nominations_share_code UNIQUE (share_code),
    CONSTRAINT uq_f_i_transfer_nominations_card UNIQUE (window_id, team_id, player_id, intent)
);
CREATE INDEX idx_f_i_transfer_nominations_board ON f_i_transfer_nominations (window_id, team_id, intent);
CREATE INDEX idx_f_i_transfer_nominations_creator ON f_i_transfer_nominations (window_id, created_by, intent);

CREATE TABLE f_i_transfer_votes (
    nomination_id UUID NOT NULL REFERENCES f_i_transfer_nominations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    value SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (nomination_id, user_id),
    CONSTRAINT ck_f_i_transfer_votes_value CHECK (value IN (1, -1))
);
CREATE INDEX idx_f_i_transfer_votes_user ON f_i_transfer_votes (user_id);

INSERT INTO f_i_transfer_windows (code, name, opens_at, closes_at)
VALUES ('2026-winter', '2026 冬窗', '2026-10-20T00:00:00+08:00', '2027-03-01T00:00:00+08:00');
