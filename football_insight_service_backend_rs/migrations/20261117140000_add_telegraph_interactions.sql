-- 每日电报条目赞/踩、单层评论，以及用户自定义头像昵称完成标记（避免将默认手机号/微信用户误判为已设昵称）。

CREATE TABLE IF NOT EXISTS f_i_user_custom_profiles (
    user_id UUID PRIMARY KEY REFERENCES f_i_users(id) ON DELETE CASCADE,
    nickname_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    avatar_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO f_i_user_custom_profiles (user_id, nickname_updated_at, avatar_updated_at, updated_at)
SELECT id, updated_at, updated_at, updated_at
  FROM f_i_users
 WHERE avatar_url IS NOT NULL
   AND btrim(avatar_url) <> ''
   AND display_name IS NOT NULL
   AND char_length(btrim(display_name)) BETWEEN 2 AND 32
   AND btrim(display_name) <> account_identifier
   AND btrim(display_name) <> '微信用户'
   AND btrim(display_name) !~ '^1[0-9]{10}$'
   AND btrim(display_name) !~* '^wx_'
ON CONFLICT (user_id) DO NOTHING;

CREATE TABLE IF NOT EXISTS f_i_telegraph_reactions (
    entry_id VARCHAR(128) NOT NULL,
    topic_id VARCHAR(128) NOT NULL,
    team_id BIGINT NOT NULL DEFAULT 77680,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    reaction VARCHAR(16) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (entry_id, user_id),
    CONSTRAINT ck_f_i_telegraph_reactions_kind CHECK (reaction IN ('like', 'dislike'))
);

CREATE INDEX IF NOT EXISTS idx_f_i_telegraph_reactions_topic_entry
    ON f_i_telegraph_reactions (team_id, topic_id, entry_id);

CREATE INDEX IF NOT EXISTS idx_f_i_telegraph_reactions_user
    ON f_i_telegraph_reactions (user_id);

CREATE TABLE IF NOT EXISTS f_i_telegraph_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id BIGINT NOT NULL DEFAULT 77680,
    topic_id VARCHAR(128) NOT NULL,
    entry_id VARCHAR(128) NOT NULL,
    user_id UUID NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    reply_to_comment_id UUID NULL REFERENCES f_i_telegraph_comments(id) ON DELETE SET NULL,
    content TEXT NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'visible',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_f_i_telegraph_comments_status CHECK (status IN ('visible', 'hidden', 'deleted')),
    CONSTRAINT ck_f_i_telegraph_comments_content_len CHECK (char_length(btrim(content)) BETWEEN 1 AND 200)
);

CREATE INDEX IF NOT EXISTS idx_f_i_telegraph_comments_entry_visible
    ON f_i_telegraph_comments (entry_id, status, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_f_i_telegraph_comments_topic_status
    ON f_i_telegraph_comments (team_id, topic_id, status);

CREATE INDEX IF NOT EXISTS idx_f_i_telegraph_comments_created_at
    ON f_i_telegraph_comments (created_at DESC, id DESC);

INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES ('telegraph_comments_enabled', 'true', '是否开启每日电报评论功能（true/false）')
ON CONFLICT (config_key) DO NOTHING;
