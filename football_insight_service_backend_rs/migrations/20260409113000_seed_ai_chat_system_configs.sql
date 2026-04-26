INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES (
    'ai_chat_model',
    'glm-5.1',
    'AI chat model name. Empty means fallback to environment default.'
)
ON CONFLICT (config_key) DO NOTHING;

INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES (
    'ai_chat_base_url',
    'https://open.bigmodel.cn/api/coding/paas/v4',
    'AI chat provider base URL. Empty means fallback to environment default.'
)
ON CONFLICT (config_key) DO NOTHING;
