# football_insight_service_backend_rs

Football Insight 的后端服务，采用 Rust + Axum + SQLx + PostgreSQL。

## 架构

- 按领域纵向切分
- 六边形架构：`domain -> ports -> application -> adapters`
- 数据库表统一使用 `f_i_` 前缀

## 本地开发

```bash
cd football_insight_service_backend_rs

cargo check
DATABASE_URL='postgresql://football_app:***@117.72.164.211:5432/football_data?sslmode=disable' cargo run --bin football_insight_service_backend_rs
```

## 环境变量

复制 `.env.example` 为 `.env`，配置：

- `PORT`
- `DATABASE_URL`
- `RUST_LOG`
- `JWT_SECRET`
- `OPENAI_API_KEY`
- `OPENAI_BASE_URL`
- `AI_CHAT_MODEL`
- `WECHAT_APP_ID`
- `WECHAT_APP_SECRET`
- `WECHAT_MINI_APP_ID`
- `WECHAT_MINI_APP_SECRET`
- `WECHAT_WEBHOOK_TOKEN`
- `WECHAT_ENCODING_AES_KEY`
- `TICKET_MONITOR_BASE_URL`

说明：

- `WECHAT_APP_ID / WECHAT_APP_SECRET` 继续服务公众号 / H5 微信链路
- `WECHAT_MINI_APP_ID / WECHAT_MINI_APP_SECRET` 服务微信小程序登录
- `OPENAI_API_KEY` 配置后会开启站内 AI 对话接口
- `OPENAI_BASE_URL` 默认为智谱 `Coding Plan` 的 OpenAI 兼容网关 `https://open.bigmodel.cn/api/coding/paas/v4`
- `AI_CHAT_MODEL` 默认为 `glm-5.1`
- 如果未单独配置小程序环境变量，后端会自动回退到 `WECHAT_APP_ID / WECHAT_APP_SECRET`
- `TICKET_MONITOR_BASE_URL` 用于余票监控 / 下一场比赛等联动能力，本地默认会回落到 `http://127.0.0.1:4000`

## 数据库

当前初始化迁移位于 `migrations/`，包含：

- 抓取任务
- 比赛
- 积分榜
- 球队榜快照/明细
- 球员榜快照/明细
- 邀请码和微信关注者

执行迁移：

```bash
DATABASE_URL='postgresql://football_app:***@117.72.164.211:5432/football_data?sslmode=disable' \
cargo run --bin run_migrations

DATABASE_URL='postgresql://football_app:***@117.72.164.211:5432/football_data?sslmode=disable' \
cargo run --bin run_migrations -- migrations/20260409001000_add_membership_tier_to_users.sql
```

## Docker 运行

本项目支持直接构建 Docker 镜像，便于后续接入 Portainer：

```bash
docker build -t football-insight-service:latest .

docker run -d \
  --name football-insight-service \
  --restart unless-stopped \
  --env-file .env \
  -p 8092:8092 \
  football-insight-service:latest
```

## 当前接口

- 健康检查
  - `/api/health`
- 系统配置
  - `/api/v1/system/public-config`
  - `/api/v1/system_config`
  - `/api/v1/system-config`
  - `/api/v1/mini-program/review-config`（兼容旧调用）
- 洞察 / 榜单 / 赛程
  - `/api/v1/live/overview`
  - `/api/v1/live/rankings`
  - `/api/v1/live/matches`
  - `/api/v1/live/team-insights`
  - `/api/v1/rounds`
  - `/api/v1/rounds/{season}/{round}/overview`
  - `/api/v1/rounds/{season}/{round}/rankings`
  - `/api/v1/rounds/{season}/{round}/matches`
- 认证
  - `/api/v1/auth/register`
  - `/api/v1/auth/login`
  - `/api/v1/auth/reset-password`
  - `/api/v1/auth/wechat/authorize`
  - `/api/v1/auth/wechat/callback`
  - `/api/v1/auth/wechat/bind`
  - `/api/v1/auth/mini-wechat/login`
  - `/api/v1/auth/mini-wechat/bind`
  - `/api/v1/auth/me`
  - `/api/v1/auth/logout`
  - `/football/wechat/webhook`
- AI
  - `/api/v1/ai/chat`
  - `/api/v1/ai/chat/stream`
- 主队助力
  - `/api/v1/support/teams`
  - `/api/v1/support/profile`
  - `/api/v1/support/favorite-team`
  - `/api/v1/support/matches/{match_id}`
  - `/api/v1/support/matches/{match_id}/votes`
- 余票监控
  - `/api/v1/ticket-watch/current-match`
  - `/api/v1/ticket-watch/matches`
  - `/api/v1/ticket-watch/regions`
  - `/api/v1/ticket-watch/matches/{match_id}/inventory`
  - `/api/v1/ticket-watch/matches/{match_id}/interests`
  - `/api/v1/ticket-watch/matches/{match_id}/interests/toggle`
- 球队战术板
  - `/api/v1/team-boards/{team_id}`
  - `/api/v1/team-boards/posts/{post_id}/comments`
  - `/api/v1/team-boards/posts/{post_id}/likes`

## System Config 审核配置接口

`GET /api/v1/system_config?version={version}`

小程序启动时用本地环境变量 `VITE_MINI_PROGRAM_VERSION` 里的当前版本号调用该接口，判断当前版本是否处于微信审核中。接口公开，不需要 JWT。

兼容别名：

- `GET /api/v1/system-config?version={version}`
- `GET /api/v1/mini-program/review-config?version={version}`

可选参数：

- `version`: 必填，小程序当前版本号，例如 `1.2.3`
- `app_id`: 可选，小程序 app id；不传时使用默认小程序配置

响应示例：

```json
{
  "mini_program_app_id": "",
  "mini_program_version": "1.2.3",
  "is_under_review": true,
  "matched": true,
  "created_at": "2026-04-24T20:00:00+00:00",
  "updated_at": "2026-04-24T20:00:00+00:00"
}
```

字段说明：

- `is_under_review=true`: 当前版本正在审核，小程序端应隐藏会员、支付等相关入口，并避免加载会员信息。
- `is_under_review=false`: 审核完成或未配置该版本，小程序端可正常展示会员信息。
- `matched=false`: 后端没有找到该版本的审核配置，默认按非审核处理。

## 数据视图

当前后端围绕两种快照视图工作：

- `live`
  - 按抓取时间形成的实时快照
- `round_final`
  - 按轮次形成的结算快照

首页摘要 `insight_summary` 当前不落表，而是在请求时根据：

- 最新快照
- 上一份快照
- 最近赛果
- 积分榜变化
- 射手榜变化

进行规则化计算。

但“球队洞察”不是这个模式。

当前 `/api/v1/live/team-insights` 的数据来源是：

- scraper 预计算
- 落表 `f_i_team_insights`
- Rust 后端读取并透传 DTO

也就是说：

- 进球贡献、助攻贡献、失球贡献不在 Rust 请求时计算
- 这些口径当前在 Python 项目 `sina_csl_scraper/src/sina_csl_scraper/team_insights.py`
- Rust 侧当前只负责查询 `f_i_team_insights`

## 认证接口

- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`
- `POST /api/v1/auth/reset-password`
- `GET /api/v1/auth/wechat/authorize`
- `GET /api/v1/auth/wechat/callback`
- `POST /api/v1/auth/wechat/bind`
- `POST /api/v1/auth/mini-wechat/login`
- `POST /api/v1/auth/mini-wechat/bind`
- `GET /api/v1/auth/me`
- `POST /api/v1/auth/logout`
- `POST /api/v1/ai/chat`
- `POST /api/v1/ai/chat/stream`

当前认证模型：

- 邀请码注册
- 用户名或手机号作为登录名
- Argon2 哈希密码
- JWT access token 鉴权

### 注册

`POST /api/v1/auth/register`

请求体：

```json
{
  "invite_code": "FI-DEV-20260405",
  "account_identifier": "footballfan",
  "password": "secret123"
}
```

字段说明：

- `invite_code`: 首次注册必填的邀请码
- `account_identifier`: 用户名或手机号
- `password`: 至少 6 位

规则：

- 支持 11 位中国大陆手机号
- 也支持长度大于 5 个字符的用户名
- 后端兼容旧字段 `phone_number`，但新请求应统一使用 `account_identifier`

### 登录

`POST /api/v1/auth/login`

请求体：

```json
{
  "account_identifier": "footballfan",
  "password": "secret123"
}
```

字段说明：

- `account_identifier`: 用户名或手机号
- `password`: 登录密码

### 微信绑定

`POST /api/v1/auth/wechat/bind`

请求体中的账号字段目前仍然是：

- `phone_number`

这条链路当前仍按手机号绑定设计，没有切到 `account_identifier`。

### 小程序微信登录

`POST /api/v1/auth/mini-wechat/login`

请求体：

```json
{
  "code": "wx-login-code"
}
```

这条链路使用微信小程序 `wx.login()` / `uni.login()` 获取的 `code`，由后端调用 `jscode2session` 换取：

- `open_id`
- `union_id`

响应有两种：

1. 已绑定微信，直接登录

```json
{
  "status": "authenticated",
  "access_token": "jwt",
  "expires_at": "2026-04-08T10:00:00Z",
  "user": {
    "id": "uuid",
    "display_name": "何止编程",
    "account_identifier": "wx_oAbc123",
    "avatar_url": "data:image/png;base64,...",
    "has_wechat_binding": true,
    "created_at": "2026-04-08T09:00:00Z"
  }
}
```

2. 未绑定微信，需要邀请码补全

```json
{
  "status": "binding_required",
  "bind_token": "bind-token",
  "expires_at": "2026-04-08T09:20:00Z",
  "display_name": null,
  "avatar_url": null
}
```

`POST /api/v1/auth/mini-wechat/bind`

请求体：

```json
{
  "bind_token": "bind-token",
  "invite_code": "FI-DEV-20260405",
  "display_name": "何止编程",
  "avatar_data_url": "data:image/png;base64,iVBORw0KGgoAAA..."
}
```

规则：

- 首次未绑定时，用户只需要：
  - 邀请码
  - 昵称
  - 头像
- 不再要求：
  - 用户名
  - 手机号
  - 密码
- 后端会自动生成一个内部 `account_identifier`，格式为 `wx_<open_id>`

### 小程序头像存储

当前小程序端不会先把头像上传到 MinIO。

而是：

- 小程序使用 `chooseAvatar` 选择头像
- 前端将头像文件读取为 `data:image/...;base64,...`
- 后端直接把这个 data URL 写入 `f_i_users.avatar_url`

也就是说：

- 当前头像以 base64 data URL 的形式落在数据库里
- 这是为了先快速打通小程序微信首登闭环，避免额外引入上传链路

当前新增表：

- `f_i_invite_codes`
- `f_i_wechat_followers`

当前新增字段：

- `f_i_users.account_identifier`
- `f_i_users.password_hash`

## 公众号消息推送

当前已支持微信公众号消息推送安全模式（XML）：

- `GET /football/wechat/webhook`
  - 微信服务器 URL 校验
- `POST /football/wechat/webhook`
  - 处理安全模式加密 XML
  - `subscribe` 事件会自动生成或复用一条邀请码
  - 文本指令 `邀请码` 会返回当前可用邀请码
  - 文本指令 `下一场id` 会调用同机 `127.0.0.1:4000` 的 `ticket-monitor-axum` `/api/matches/current-standard`，并返回 `match_id`
  - 通过被动回复把邀请码发回给用户

## 生产部署

当前生产环境后端使用 systemd 管理：

- 服务名：`football-insight.service`
- unit 模板：`deploy/football-insight.service`
- 项目目录：`/root/projects/football_insight/football_insight_service_backend_rs`
- 环境变量文件：`/root/projects/football_insight/football_insight_service_backend_rs/.env`
- 监听端口：`8092`

首次安装或更新 unit 文件：

```bash
ssh jd
cd /root/projects/football_insight/football_insight_service_backend_rs
cp deploy/football-insight.service /etc/systemd/system/football-insight.service
systemctl daemon-reload
systemctl enable football-insight.service
systemctl restart football-insight.service
```

常规后端发布流程：

```bash
ssh jd
cd /root/projects/football_insight
git pull --ff-only
cd football_insight_service_backend_rs
cargo build --release

# 如果本次发布包含新 migration，再执行
cargo run --release --bin run_migrations

systemctl restart football-insight.service
systemctl status football-insight.service --no-pager
curl -i http://127.0.0.1:8092/
curl -k -i https://match.oryjk.cn/api/v1/ticket-watch/current-board
```

后端代码发布通常不需要重启 Nginx。只有修改 Nginx 配置时才执行 `nginx -t` 后 reload/restart Nginx。

不要在生产上用裸 `cargo run`、前台进程或只依赖 SSH 会话的后台进程托管后端。systemd 会在 SSH 断开后继续托管进程，并在进程异常退出时按 unit 的 `Restart=always` 自动拉起。

## 生产日志

应用代码会把 tracing 日志写入项目目录下的 `logs/`：

- 当前日志：`/root/projects/football_insight/football_insight_service_backend_rs/logs/app.log`
- 滚动日志：`/root/projects/football_insight/football_insight_service_backend_rs/logs/app.<timestamp>.log`

日志达到 10MB 后会自动滚动，当前代码最多保留 100 个滚动文件。

systemd unit 会丢弃 stdout，并把 stderr 写入 journal：

- `StandardOutput=null`
- `StandardError=journal`

journal 用作进程启动失败、panic 或早期 stderr 输出的兜底排查入口，不是业务日志的首选入口。

常用查看方式：

```bash
ssh jd 'tail -n 100 -f /root/projects/football_insight/football_insight_service_backend_rs/logs/app.log'
journalctl -u football-insight.service -n 100 --no-pager
journalctl -u football-insight.service -f
```

服务状态查看：

```bash
ssh jd 'systemctl status football-insight.service --no-pager'
```

历史遗留的 `service.log` 不再继续增长，不再作为当前日志入口。

## 微信接入知识

当前项目里和微信相关的能力，要分成 3 类来看，不要混在一起：

### 1. 公众号消息推送

这是“微信把事件主动推给我们”的模式，例如：

- 用户关注 `subscribe`
- 用户取消关注 `unsubscribe`
- 用户发送文本消息

这类能力依赖：

- 消息推送 URL
- `WECHAT_WEBHOOK_TOKEN`
- `WECHAT_ENCODING_AES_KEY`

这条链路**不依赖** `WECHAT_APP_ID` / `WECHAT_APP_SECRET`。

### 2. 公众号网页授权登录

这是“H5 主动跳到微信授权页，再带 `code` 回来”的模式。

这类能力会用到：

- `WECHAT_APP_ID`
- `WECHAT_APP_SECRET`

但是否真的能授权成功，不只取决于代码参数，还取决于公众号**类型和权限**。

当前已知限制：

- 个人主体、未认证的订阅号，通常**不具备**当前需要的网页授权 scope 能力
- 即使把 scope 从 `snsapi_userinfo` 改成 `snsapi_base`，仍然可能被微信直接拦截

当前项目已经把 OAuth scope 调整为：

- `snsapi_base`

这意味着：

- 如果公众号本身具备网页授权能力，后端只依赖 `openid` / `unionid` 做识别和绑定
- 不再依赖 `sns/userinfo` 去取昵称和头像

但如果公众号本身没有该能力，仍然会在微信授权页报：

- `Scope 参数错误或没有 Scope 权限`

### 3. 主动调用微信 API

这里才需要先用：

- `WECHAT_APP_ID`
- `WECHAT_APP_SECRET`

去换取公众号级 `access_token`。

`access_token` 的作用是：

- 让后端可以代表公众号去主动调用微信接口

典型用途包括：

- 管理菜单
- 获取部分公众号侧资料
- 调用其他公众号开放接口

注意：

- 接收关注事件本身，不需要公众号 `access_token`
- `AppID/AppSecret` 是开发身份凭据，不等于自动拥有 OAuth 登录权限

## 当前产品侧建议

基于目前公众号能力边界，当前更稳定的产品路径是：

- 保留“关注公众号 -> 自动发邀请码”
- H5 继续使用“邀请码注册 + 手机号密码登录”
- 不把公众号 OAuth 登录当成当前主登录方式

如果后面切到**已认证服务号**，再重新启用公众号网页授权登录会更稳。

## AI 对话接口

`POST /api/v1/ai/chat`

说明：

- 需要登录态，请求头必须带 `Authorization: Bearer <access_token>`
- 当前通过 `rig-core` 接入智谱 GLM 的 OpenAI 兼容对话接口
- 如果未配置 `OPENAI_API_KEY`，接口会返回 `503`
- 当前接口只负责“单轮消息 + 有限历史消息”对话，不落库存档

请求体：

```json
{
  "message": "帮我总结一下这轮中超榜首走势",
  "history": [
    {
      "role": "user",
      "content": "我想看看榜首变化"
    },
    {
      "role": "assistant",
      "content": "你更想看积分榜还是球队走势？"
    }
  ]
}
```

字段规则：

- `message`: 当前用户输入，必填，最大 4000 字符
- `history`: 可选，只支持 `user` 和 `assistant` 两种角色
- 后端会自动截取最近 20 条非空历史消息送给模型

响应体：

```json
{
  "model": "glm-5.1",
  "reply": "如果只看当前榜首走势，成都蓉城和上海申花的竞争会是最值得关注的一段。"
}
```

示例：

```bash
curl 'http://127.0.0.1:8092/api/v1/ai/chat' \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <access_token>' \
  -d '{
    "message": "帮我介绍一下这个产品能看什么",
    "history": []
  }'
```

## 相关项目

- 总说明：
  [football_insight/README.md](../README.md)
- 前端：
  [football_insight_mini/README.md](../football_insight_mini/README.md)
- Python 抓取器：
  [sina_csl_scraper/README.md](../sina_csl_scraper/README.md)
