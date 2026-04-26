CREATE TABLE IF NOT EXISTS f_i_mini_program_review_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mini_program_app_id VARCHAR(128) NOT NULL DEFAULT '',
    mini_program_version VARCHAR(32) NOT NULL DEFAULT '',
    is_under_review BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (mini_program_app_id, mini_program_version)
);

CREATE INDEX IF NOT EXISTS idx_f_i_mini_program_review_configs_is_under_review
    ON f_i_mini_program_review_configs (is_under_review);

INSERT INTO f_i_mini_program_review_configs (
    mini_program_app_id,
    mini_program_version,
    is_under_review
)
VALUES ('', '', FALSE)
ON CONFLICT (mini_program_app_id, mini_program_version) DO NOTHING;
