-- 支持按父评论筛选每日电报根评论与子回复线程（X 风格评论详情时间线）。
CREATE INDEX IF NOT EXISTS idx_f_i_telegraph_comments_parent_visible
    ON f_i_telegraph_comments (entry_id, reply_to_comment_id, status, created_at DESC, id DESC);
