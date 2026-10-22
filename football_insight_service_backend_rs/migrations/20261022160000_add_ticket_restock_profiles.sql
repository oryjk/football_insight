-- 「最佳盯盘时段」离线统计快照：每场已结束主场一行，由 ticket-monitor 的离线批处理
-- （peiqian 上的 ticket-restock-profiles systemd 任务）按 match_id 幂等写入，football 后端只读。
-- since_at 是统计起点（开售后 10 分钟，北京时间换算成 timestamptz）；hourly 固定 24 个北京小时桶。
CREATE TABLE f_i_ticket_restock_profiles (
    match_id BIGINT PRIMARY KEY,
    schema_version SMALLINT NOT NULL,
    stat_unit TEXT NOT NULL,
    since_at TIMESTAMPTZ NOT NULL,
    total_events BIGINT NOT NULL,
    appear_events BIGINT NOT NULL,
    up_events BIGINT NOT NULL,
    seats_with_events BIGINT NOT NULL,
    hourly BIGINT[] NOT NULL,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_f_i_ticket_restock_profiles_hourly CHECK (cardinality(hourly) = 24),
    CONSTRAINT ck_f_i_ticket_restock_profiles_counts CHECK (
        total_events >= 0 AND appear_events >= 0 AND up_events >= 0
        AND appear_events + up_events = total_events
        AND seats_with_events >= 0 AND seats_with_events <= total_events
    )
);
