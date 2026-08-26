CREATE TABLE IF NOT EXISTS f_i_mini_review_statuses (
    id BIGSERIAL PRIMARY KEY,
    project_code VARCHAR(64) NOT NULL,
    version VARCHAR(32) NOT NULL,
    version_code BIGINT NOT NULL,
    is_reviewing BOOLEAN NOT NULL DEFAULT TRUE,
    status_text VARCHAR(255) NOT NULL DEFAULT '正在审核',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (project_code, version)
);

CREATE INDEX IF NOT EXISTS idx_f_i_mini_review_statuses_project_latest
    ON f_i_mini_review_statuses (project_code, version_code DESC);
