-- Logged-in readers per topic. Anonymous visits stay only in f_i_daily_topic_visitors.
CREATE TABLE f_i_daily_topic_readers (
    team_id BIGINT NOT NULL,
    topic_id TEXT NOT NULL,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    first_read_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (team_id, topic_id, user_id),
    FOREIGN KEY (team_id, topic_id) REFERENCES f_i_daily_topic_metrics(team_id, topic_id) ON DELETE CASCADE
);

CREATE INDEX idx_f_i_daily_topic_readers_order
    ON f_i_daily_topic_readers (team_id, topic_id, first_read_at);
