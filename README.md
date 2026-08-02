# Football Insight

一个面向中国足球用户的“数据洞察”产品实验项目，当前一期 MVP 聚焦中超联赛。

当前 monorepo 下包含 4 个子项目：

- `sina_csl_scraper`
  - Python + `uv`
  - 负责抓取新浪移动端数据、下载球队/球员头像、上传 MinIO、写入 PostgreSQL
- `football_insight_service_backend_rs`
  - Rust + Axum + SQLx
  - 负责读取 PostgreSQL 中的结构化数据，对前端提供 API
- `football_insight_mini`
  - uni-app + Vue 3 + TypeScript + Bun
  - 负责微信小程序，并支持编译为 H5
- `football_insight_admin_android`
  - Kotlin + Jetpack Compose + Material 3
  - 原生 Android 管理端，只调用 `/api/v1/admin/**`

## 当前能力

- 联赛总览
- 实时脉冲 / 轮次复盘 双视图
- 积分榜、球队榜、球员榜
- 最近赛果
- 首页 `insight_summary` 规则化摘要
- 洞察页球队贡献分析
  - 进球贡献
  - 助攻贡献
  - 失球贡献
- 球队/球员头像上传到 MinIO 并通过公开 URL 展示

## 数据链路

```text
新浪移动端接口
  -> sina_csl_scraper
  -> PostgreSQL (football_data, f_i_* tables)
  -> football_insight_service_backend_rs
  -> football_insight_mini / football_insight_admin_android
```

当前职责边界：

- Python 项目负责“写”
- Rust 项目负责“读”
- 前端只消费 Rust API，不直接访问新浪接口和数据库

关于“归因 / 贡献”需要特别说明：

- 洞察页里的贡献分析不是请求时现算
- 预计算逻辑在 Python 项目 `sina_csl_scraper`
- 入口代码在 `src/sina_csl_scraper/team_insights.py`
- scraper 同步后会把结果写入 PostgreSQL 的 `f_i_team_insights`
- Rust 后端只读取 `f_i_team_insights`
- 前端当前通过 `/api/v1/live/team-insights` 获取这些已经算好的结果

## 目录结构

```text
football_insight/
├── DEPLOYMENT.md
├── README.md
├── football_insight_mini/
├── football_insight_admin_android/
├── football_insight_service_backend_rs/
└── sina_csl_scraper/
```

## 开发环境

- PostgreSQL
- MinIO
- Bun
- Rust
- uv / Python 3.13

## 部署说明

生产环境部署 SOP 见：

- [DEPLOYMENT.md](DEPLOYMENT.md)

## 数据抓取补充

如果后续需要用浏览器方式分析页面数据，优先看这份说明：

- [docs/playwright-data-extraction.md](docs/playwright-data-extraction.md)

这份文档包含：

- 用 Playwright 监听页面真实接口
- 用 Playwright 直接读 DOM
- 先点击/切换再继续抓数据
- 什么时候才应该走截图 + OCR

## 快速启动

### 1. 先抓取并写库

```bash
cd sina_csl_scraper

FI_DATABASE_URL='postgresql://football_app:***@117.72.164.211:5432/football_data?sslmode=disable' \
FI_MINIO_ENDPOINT='https://oryjk.cn:82/minio' \
FI_MINIO_ACCESS_KEY='***' \
FI_MINIO_SECRET_KEY='***' \
FI_MINIO_BUCKET='football-insight' \
FI_MINIO_PREFIX='summary' \
uv run sina-csl-scraper scrape --season 2026 --upload-avatars --write-db
```

### 2. 启动 Rust 后端

```bash
cd football_insight_service_backend_rs

DATABASE_URL='postgresql://football_app:***@117.72.164.211:5432/football_data?sslmode=disable' \
JWT_SECRET='replace-with-a-long-random-secret' \
PORT=8092 \
cargo run
```

### 3. 启动前端 H5

```bash
cd football_insight_mini

bun install
bun run dev:h5
```

### 4. 启动小程序前端

```bash
cd football_insight_mini

bun install
bun run dev:mp-weixin
```

然后用微信开发者工具打开：

- `football_insight_mini/dist/dev/mp-weixin`

### 5. 构建 Android 管理端

```bash
cd football_insight_admin_android
ANDROID_HOME=/home/betalpha/Android/Sdk ./gradlew testDebugUnitTest lintDebug assembleDebug
```

管理端内网 Debug 包默认使用 local233 管理 API `http://172.16.60.233:18092/`，Release 构建默认使用生产 API `https://match.oryjk.cn/`。模拟器访问本机后端时使用 `http://10.0.2.2:18092/`；也可通过 Gradle 属性 `FOOTBALL_ADMIN_API_BASE_URL` 覆盖。App 会通过 `/api/v1/system/public-config` 的 JSON 响应识别正确后端，不会只凭 HTTP 200 接受错误页面。

本机测试环境启动：

```bash
./deploy/local-test/up.sh
```

后端和 Redis 运行在 local233，后端通过私有 SSH 隧道直接连接 `peiqian` 上的生产 `football_data`，并在启动时自动执行 migration。管理员：`admin` / `admin123`。会员调整、禁用/恢复等写操作会直接修改生产用户数据。

## 账号系统说明

当前前端已增加 `/user` 页面，采用：

- 邀请码注册
- 用户名或手机号作为登录名
- 密码登录
- JWT access token 鉴权

当前后端接口：

- `POST /api/v1/auth/register`
- `POST /api/v1/auth/login`
- `GET /api/v1/auth/me`
- `POST /api/v1/auth/logout`
- `GET /football/wechat/webhook`
- `POST /football/wechat/webhook`

关键环境变量：

- `JWT_SECRET`

本地联调用的邀请码：

- `FI-DEV-20260405`

注意：

- 邀请码注册成功后会返回 JWT 并自动登录
- 一个邀请码当前只使用一次
- 当前演示账号可用手机号示例：`13800138000`
- 公众号关注事件会自动生成邀请码并通过消息推送返回给用户

登录/注册请求字段：

- `POST /api/v1/auth/register`
  - `invite_code: string`
  - `account_identifier: string`
  - `password: string`
- `POST /api/v1/auth/login`
  - `account_identifier: string`
  - `password: string`

`account_identifier` 规则：

- 支持 11 位中国大陆手机号
- 也支持长度大于 5 个字符的用户名
- 后端暂时兼容旧字段 `phone_number`，但新请求默认应使用 `account_identifier`

## 小程序微信登录补充

当前小程序版本位于：

- [football_insight_mini](football_insight_mini/README.md)

小程序端新增了独立微信登录链路：

- 已绑定微信：直接登录
- 首次未绑定：填写邀请码，并补充头像和昵称后完成注册 / 绑定

这条链路当前不要求：

- 用户名
- 手机号
- 密码

头像当前会由小程序端转成 `data:image/...;base64,...`，再由后端写入 `f_i_users.avatar_url`。

## 数据模型说明

数据库统一使用 `f_i_` 前缀，表示 `football_insight`。

当前重点表：

- `f_i_scrape_runs`
- `f_i_teams`
- `f_i_players`
- `f_i_matches`
- `f_i_standings`
- `f_i_ranking_categories`
- `f_i_team_ranking_snapshots`
- `f_i_team_ranking_entries`
- `f_i_player_ranking_snapshots`
- `f_i_player_ranking_entries`
- `f_i_team_insights`

快照模型支持两种视图：

- `live`
  - 按抓取时间形成的实时快照
- `round_final`
  - 按轮次结算形成的复盘快照

## 当前 API

后端当前已提供：

- `/api/v1/live/overview`
- `/api/v1/live/rankings`
- `/api/v1/live/matches`
- `/api/v1/live/team-insights`
- `/api/v1/rounds`
- `/api/v1/rounds/{season}/{round}/overview`
- `/api/v1/rounds/{season}/{round}/rankings`
- `/api/v1/rounds/{season}/{round}/matches`

## 后续方向

- 首页补强“比赛影响摘要 / 上升最快 / 下降最快”
- 球队页
- 球员页
- 比赛页
- 更多洞察型卡片
