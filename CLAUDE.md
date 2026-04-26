# Football Insight - Project Guide

面向中国足球用户的"数据洞察"产品实验项目，当前聚焦中超联赛（CSL）。

## Architecture

三个独立子项目，数据单向流动：

```
新浪移动端 API → sina_csl_scraper (Python/写) → PostgreSQL → football_insight_service_backend_rs (Rust/读) → football_insight_mini (uni-app/展示)
```

- **sina_csl_scraper** — 负责抓取、写库、上传头像到 MinIO
- **football_insight_service_backend_rs** — 负责读库、对外提供 REST API
- **football_insight_mini** — 负责微信小程序，并支持编译为 H5，消费 Rust API

## Tech Stack

| 子项目 | 技术栈 | 包管理 | 测试 |
|--------|--------|--------|------|
| frontend | Vue 3 + TypeScript + Vite + Vant + Pinia + TanStack Query + Bun | bun | Playwright |
| backend | Rust + Axum + SQLx + tokio + JWT + argon2 | cargo | cargo test + tower |
| scraper | Python 3.12+ + requests + psycopg + MinIO + typer | uv | pytest |

## Commands

### Frontend (`football_insight_mini/`)

```bash
bun install          # 安装依赖
bun run dev:h5       # H5 开发服务器
bun run build:h5     # H5 生产构建，输出到 dist/build/h5/
bun run dev:mp-weixin   # 微信小程序开发
```

### Backend (`football_insight_service_backend_rs/`)

```bash
cargo test           # 运行测试
cargo run            # 启动服务 (默认 8080，通过 PORT 环境变量覆盖，生产用 8092)
```

### Scraper (`sina_csl_scraper/`)

```bash
uv run pytest                                          # 运行测试
uv run sina-csl-scraper scrape --season 2026 --write-db  # 抓取数据写入数据库
uv run sina-csl-scraper scrape --season 2026 --upload-avatars --write-db  # 含头像上传
```

## Environment Variables

### Backend

| 变量 | 说明 | 必须 |
|------|------|------|
| `DATABASE_URL` | PostgreSQL 连接串 | 是 |
| `JWT_SECRET` | JWT 签名密钥 | 是 |
| `PORT` | 监听端口（默认 8080） | 否 |
| `WECHAT_APP_ID` | 微信公众号 AppID | 是 |
| `WECHAT_APP_SECRET` | 微信公众号 Secret | 是 |
| `WECHAT_WEBHOOK_TOKEN` | 微信消息校验 Token | 是 |
| `WECHAT_ENCODING_AES_KEY` | 微信消息加密 Key | 是 |

### Scraper

| 变量 | 说明 |
|------|------|
| `FI_DATABASE_URL` | PostgreSQL 连接串 |
| `FI_MINIO_ENDPOINT` | MinIO 地址 |
| `FI_MINIO_ACCESS_KEY` | MinIO Access Key |
| `FI_MINIO_SECRET_KEY` | MinIO Secret Key |
| `FI_MINIO_BUCKET` | MinIO Bucket 名 |
| `FI_MINIO_PREFIX` | MinIO 对象前缀 |

## API Routes

后端路由前缀 `/api/v1`。

**Insight 数据接口：**
- `GET /live/overview` — 联赛实时总览
- `GET /live/rankings` — 实时榜单
- `GET /live/matches` — 实时赛果
- `GET /rounds` — 可用轮次列表
- `GET /rounds/{season}/{round}/overview` — 轮次复盘总览
- `GET /rounds/{season}/{round}/rankings` — 轮次复盘榜单
- `GET /rounds/{season}/{round}/matches` — 轮次复盘赛果

**Auth 接口：**
- `POST /auth/register` — 邀请码注册
- `POST /auth/login` — 手机号密码登录
- `GET /auth/me` — 当前用户信息（需 Bearer token）
- `POST /auth/logout` — 登出
- `GET /auth/wechat/authorize` — 微信 OAuth 授权 URL
- `POST /auth/wechat/bind` — 绑定微信账号
- `GET /football/wechat/webhook` — 微信消息校验
- `POST /football/wechat/webhook` — 微信消息接收

## Frontend Structure

前端代码位于 `football_insight_mini/src/`，由 uni-app 同时产出微信小程序和 H5。

## Backend Structure (Hexagonal Architecture)

```
football_insight_service_backend_rs/src/
├── main.rs           # 入口：加载配置、建连接池、启动 Axum 服务
├── app.rs            # Router 组装、依赖注入
├── config.rs         # 环境变量配置 (AppConfig)
├── auth/             # 认证模块
│   ├── adapters/     # 适配器层（PostgreSQL 仓库、JWT/Argon2 端口、微信集成）
│   ├── application/  # 用例层（注册、登录、登出、微信绑定等）
│   ├── domain/       # 领域模型
│   └── ports/        # 端口接口
├── insight/          # 数据洞察模块
│   ├── adapters/     # PostgreSQL 查询仓库、Web handlers/routes/dto
│   ├── application/  # 用例层（overview, rankings, matches 等查询）
│   ├── domain/       # 领域模型
│   └── ports/        # 端口接口
└── health/           # 健康检查模块
```

- 每个业务模块遵循六边形架构：`domain` → `ports` → `application` → `adapters`
- 数据库查询在 `adapters/persistence/` 中通过 SQLx 实现
- Web 层在 `adapters/web/` 中定义 Axum handlers、routes、DTO

## Database

- PostgreSQL，database: `football_data`
- 表前缀统一为 `f_i_`（football_insight）
- Migrations 在 `football_insight_service_backend_rs/migrations/`

核心表：
- `f_i_scrape_runs` — 抓取运行记录
- `f_i_teams` / `f_i_players` — 球队/球员
- `f_i_matches` — 比赛
- `f_i_standings` — 积分榜
- `f_i_ranking_categories` — 榜单类别
- `f_i_team_ranking_snapshots` / `f_i_team_ranking_entries` — 球队榜单快照
- `f_i_player_ranking_snapshots` / `f_i_player_ranking_entries` — 球员榜单快照

快照类型：`live`（实时）和 `round_final`（轮次复盘）

## Deployment

- 生产服务器：`jd`
- 后端目录：`/root/projects/football_insight_service_backend_rs`
- 前端静态文件：`/root/docker_data/nginx/html/football/`
- Nginx 运行在 Docker 中
- 修改 Nginx 配置时只改 football 相关路径，不要动其他服务

## Conventions

- 前端必须用 `bun`，后端用 `cargo`，scraper 用 `uv`
- 不要提交 `.env`、密钥、证书到 git
- 本地开发邀请码：`FI-DEV-20260405`
- 认证方式：邀请码注册 + 手机号密码登录；微信 OAuth 需要已认证服务号
- 后端 API 前缀统一 `/api/v1`

## Data Flow Details

**Scraper 数据流：**
`cli.py` → `sync.py` → `client.py`（调用新浪 API）→ `postgres_repository.py`（写库）→ `assets.py`（上传头像到 MinIO）

**Backend 数据流：**
Axum route → handler → use case → port (trait) → adapter (SQLx query) → PostgreSQL

**Frontend 数据流：**
uni-app page/component → API module → Rust backend
