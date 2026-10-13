-- Metrics are independent of generated batches, preserving counts across republication.
CREATE TABLE f_i_daily_topic_metrics (
    team_id BIGINT NOT NULL CHECK (team_id > 0),
    topic_id TEXT NOT NULL,
    base_heat BIGINT NOT NULL DEFAULT 0 CHECK (base_heat BETWEEN 0 AND 9007199254740991),
    view_count BIGINT NOT NULL DEFAULT 0 CHECK (view_count BETWEEN 0 AND 9007199254740991),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (team_id, topic_id)
);

CREATE TABLE f_i_daily_topic_visitors (
    team_id BIGINT NOT NULL,
    topic_id TEXT NOT NULL,
    visitor_hash TEXT NOT NULL CHECK (visitor_hash ~ '^[0-9a-f]{64}$'),
    last_counted_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (team_id, topic_id, visitor_hash),
    FOREIGN KEY (team_id, topic_id) REFERENCES f_i_daily_topic_metrics(team_id, topic_id) ON DELETE CASCADE
);
