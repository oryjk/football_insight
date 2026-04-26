INSERT INTO f_i_system_configs (
    config_key,
    config_value,
    description,
    updated_at
)
VALUES
    ('ai_chat_web_search_enabled', 'true', 'AI 对话是否启用联网搜索工具', NOW()),
    ('ai_chat_web_search_engine', 'search_std', 'AI 对话联网搜索引擎', NOW()),
    ('ai_chat_web_search_count', '5', 'AI 对话单次联网搜索结果数量', NOW())
ON CONFLICT (config_key) DO NOTHING;
