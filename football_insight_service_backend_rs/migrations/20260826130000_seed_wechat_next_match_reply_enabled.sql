INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES (
    'wechat_next_match_reply_enabled',
    'false',
    '微信公众号“下一场id/下一场”指令是否返回当前比赛 id；关闭时统一回复暂未获取。'
)
ON CONFLICT (config_key) DO NOTHING;
