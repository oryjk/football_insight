INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES (
    'wechat_invite_reply_template',
    '感谢关注【何止编程】。

你的专属邀请码：{invite_code}

产品入口：
微信搜索小程序【洞察足球集散地】

使用方式：
1. 微信搜索小程序【洞察足球集散地】
2. 使用邀请码完成注册
3. 注册后即可开始使用',
    '微信公众号邀请码回复模板，必须包含 {invite_code} 占位符。'
)
ON CONFLICT (config_key) DO NOTHING;
