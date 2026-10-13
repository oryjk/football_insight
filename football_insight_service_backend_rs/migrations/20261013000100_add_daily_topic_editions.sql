CREATE TABLE f_i_daily_topic_editions (
    id TEXT PRIMARY KEY,
    team_id BIGINT NOT NULL,
    edition_date DATE NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    payload JSONB NOT NULL,
    source_fingerprint TEXT NOT NULL,
    generator_version TEXT NOT NULL
);

CREATE INDEX idx_f_i_daily_topic_editions_team_published
    ON f_i_daily_topic_editions (team_id, published_at DESC);
