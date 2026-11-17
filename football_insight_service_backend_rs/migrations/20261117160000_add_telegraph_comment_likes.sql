-- 每日电报单条评论点赞表（X 时间线风格评论互动）。
CREATE TABLE IF NOT EXISTS f_i_telegraph_comment_likes (
    comment_id UUID NOT NULL REFERENCES f_i_telegraph_comments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (comment_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_f_i_telegraph_comment_likes_user
    ON f_i_telegraph_comment_likes (user_id);
