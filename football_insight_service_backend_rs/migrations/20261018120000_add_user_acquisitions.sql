CREATE TABLE f_i_user_acquisitions (
    user_id UUID PRIMARY KEY REFERENCES f_i_users(id) ON DELETE CASCADE,
    src VARCHAR(32) NOT NULL CHECK (src ~ '^[a-z0-9_-]+$'),
    cid VARCHAR(64) CHECK (cid ~ '^[a-z0-9_-]+$'),
    landing_scene INTEGER CHECK (landing_scene BETWEEN 1 AND 65535),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_f_i_user_acquisitions_source_content
    ON f_i_user_acquisitions (src, cid, created_at);
