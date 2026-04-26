UPDATE f_i_system_configs
   SET config_value = '【足球洞察】专属邀请码

邀请码：{invite_code}

使用方式：
1. 微信搜索小程序【洞察足球集散地】
2. 输入邀请码完成注册
3. 注册后即可开始使用

常用指令：
- 回复“邀请码”或“1”可再次获取邀请码
- 回复“下一场”或“2”可查询当前比赛 id
- 回复“下一场id”或“3”可查询当前比赛 id',
       description = '微信公众号邀请码回复模板，必须包含 {invite_code} 占位符。'
 WHERE config_key = 'wechat_invite_reply_template';
