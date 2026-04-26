# Football Insight Workspace Guide

本目录下是一个独立于报名系统主业务的足球数据产品实验项目。当前是 monorepo，包含 3 个子项目：

- `football_insight_service_backend_rs`
  - Rust + Axum + SQLx + PostgreSQL
  - 负责读库、对外提供 API
- `football_insight_mini`
  - uni-app + Vue 3 + TypeScript + Bun
  - 负责微信小程序，并支持编译为 H5
- `sina_csl_scraper`
  - Python + uv
  - 负责抓取新浪体育移动端数据、上传头像、写入 PostgreSQL

## 当前产品方向

产品定位是“足球数据洞察”，不是报名系统后台。

当前已经落地的核心链路：

- 新浪接口
  -> Python scraper
  -> PostgreSQL `football_data`
  -> Rust API
  -> 小程序 / H5 前端

关于“洞察 / 归因 / 贡献”这条链路，当前必须记住：

- 进球贡献、助攻贡献、失球贡献的计算放在 `sina_csl_scraper`
- 不是前端请求时现算，也不是 Rust 后端现算
- scraper 每次同步后会按“球队维度”预计算
- 结果会写入 `f_i_team_insights`
- Rust 后端当前只负责读取 `f_i_team_insights` 并通过 `/api/v1/live/team-insights` 返回给前端

## 当前数据库

- PostgreSQL
- database: `football_data`
- 表前缀统一为 `f_i_`，代表 `football_insight`

## 当前部署约定

- 生产服务器：`jd`
- 生产 monorepo 目录：`/root/projects/football_insight`
- 后端项目目录：`/root/projects/football_insight/football_insight_service_backend_rs`
- 前端静态目录：`/root/docker_data/nginx/html/football/`
- 后端生产优先使用 Docker 容器部署：`football-insight-service-backend-rs`
- systemd unit `football-insight.service` 保留为备用部署方式，不是当前首选
- Nginx 在 Docker 中运行

修改 `jd` 上 Nginx 时必须注意：

- 只改 football 相关路径
- 不要动其他服务配置
- 如果发现路由冲突，先告诉用户，不要直接改

## 微信相关现状

- 公众号消息推送 webhook 已接通
- `subscribe` 事件会生成邀请码并被动回复
- 当前个人主体公众号不适合作为 H5 OAuth 登录入口
- 如要稳定支持微信网页授权登录，需要“已认证服务号”

所以当前稳定登录方式应优先使用：

- 邀请码注册
- 手机号密码登录

## 开发约定

- 前端必须使用 `bun`
- 后端用 `cargo`
- scraper 用 `uv`
- 任何 `.env`、密钥、证书都不要提交到 git
- 只要修改了后端代码，本地验证前必须重启本地后端进程；不要在旧进程上直接验证新代码行为
- 回报状态时要明确区分：
  - 本地后端已重启
  - 生产后端未部署/未重启
  - 生产后端已部署并重启

## 常用命令

### 后端

```bash
cd football_insight_service_backend_rs
cargo test
cargo run
```

### 前端

```bash
cd football_insight_mini
bun install
bun run dev:h5
bun run build:h5
bun run dev:mp-weixin
```

### 抓取器

```bash
cd sina_csl_scraper
uv run pytest
uv run sina-csl-scraper scrape --season 2026 --write-db
```

## 优先检查项

如果新开窗口接手这个项目，先看：

1. 本文件
2. `README.md`
3. `DEPLOYMENT.md`
4. 各子项目 README
