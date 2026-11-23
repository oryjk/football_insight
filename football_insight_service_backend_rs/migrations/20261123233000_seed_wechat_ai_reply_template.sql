INSERT INTO f_i_system_configs (config_key, config_value, description)
VALUES (
    'wechat_ai_reply_template',
    '【何止编程 · AI 工程工作流实战清单】

感谢来聊！这次一个人用 AI 把「记金（Sumly）」从 0 磨到 App Store 上线（原生 iOS + Go 后端 + React 配置后台），核心踩坑与落地心法总结为 4 条：

1. 拆角色，别塞一个对话框
用多 Agent 分工（架构 / 客户端 / 后端 / 独立审校 / 发布）+ Git Worktree 隔离并行改代码，避免上下文漂移。

2. 先搭护栏，再让 AI 写业务
严格类型与分层架构（Swift 6 / Clean Architecture）+ 380+ 自动化测试 + 模拟器截屏视觉验收，把 AI 关进可验证的铁轨里。

3. 文档即上下文，规范即提示词
把项目约定、边界口径、发布红线写进仓库 AGENTS.md，让每个新开的 Agent 开局就懂你的工程标准。

4. 人定品味，AI 扛执行
把精力花在产品取舍与体验细节，用 AI 把验证周期从两周压缩到半小时。

——————
交流与 AI 咨询：
如果你正在探索团队/个人 AI Coding 工作流与多 Agent 落地，或想用 AI 低成本把产品想法从 0 到 1 做出上线级 MVP，欢迎扫码加我微信（备注「AI交流」优先通过）：
<a href="https://oryjk.cn:82/minio/football-insight/summary/student/contact-wechat-740aed5038f2113f.jpg">点击查看联系微信二维码</a>

也可直接在公众号回复你的场景，看到必回。',
    '微信公众号回复“AI / AI咨询 / AI工作流”的自动回复模板；留空时回退默认内置文案。'
)
ON CONFLICT (config_key) DO NOTHING;
