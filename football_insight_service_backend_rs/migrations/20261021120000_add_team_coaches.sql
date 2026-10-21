-- 各队现任主教练（新浪 /op/api/team 的 active_manager），供转会许愿「走 / 留」提名。
-- 新浪不给教练 ID：以英文名为自然键，coach_id 从 9000000001 起编号，与新浪球员 pid 不重叠，
-- 因此可直接写入 f_i_transfer_nominations.player_id。教练换队时同一行改 team_id，旧卡标记为已离队。
CREATE SEQUENCE f_i_team_coaches_coach_id_seq START WITH 9000000001;

CREATE TABLE f_i_team_coaches (
    coach_id BIGINT PRIMARY KEY DEFAULT nextval('f_i_team_coaches_coach_id_seq'),
    coach_key TEXT NOT NULL,
    coach_name TEXT NOT NULL,
    team_id BIGINT NOT NULL REFERENCES f_i_teams(team_id),
    active BOOLEAN NOT NULL DEFAULT TRUE,
    avatar_source_url TEXT,
    avatar_object_name TEXT,
    avatar_storage_url TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_f_i_team_coaches_key UNIQUE (coach_key),
    CONSTRAINT ck_f_i_team_coaches_id CHECK (coach_id >= 9000000001)
);
ALTER SEQUENCE f_i_team_coaches_coach_id_seq OWNED BY f_i_team_coaches.coach_id;

-- 每队至多一位现任主教练。
CREATE UNIQUE INDEX uq_f_i_team_coaches_active_team ON f_i_team_coaches (team_id) WHERE active;
