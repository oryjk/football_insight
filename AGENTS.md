# Football Insight Workspace Guide

本目录下是一个已上线运行的足球数据产品（线上入口 `match.oryjk.cn`，部署拓扑见下方「当前部署约定」），前端架构参考了报名系统项目。当前是 monorepo，包含 3 个子项目：

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

## 统计口径

- 所有抢票、余票、回流相关统计，统一只统计“抢票开始后 10 分钟之后”的数据
- 前端、后端、抓取器如需按比赛统计库存/回流，都必须使用 `sale_start_at + 10 minutes` 作为起算时间
- 如果某场比赛缺少 `sale_start_at`，必须报错或跳过该统计入口；不要默默改成全量统计

## 当前部署约定

- 线上域名入口：`jd`（`match.oryjk.cn` DNS 解析到 jd 公网 IP 117.72.164.211）；jd 的 Nginx 把 `/api/v1/`、`/football/wechat/webhook` 反代到 `peiqian`（内网 10.8.10.2:8092），`/football/` 前端静态资源由 jd 本地直接服务
- 真正承接线上 API / 公众号 webhook 流量的后端是 `peiqian` 上的 Docker 容器；发版 `peiqian` 即对线上生效。jd 上也保留一个后端容器但不在流量路径上（备用，无需每次同步发版）
- 生产数据库 `football-data-postgres` 运行在 `peiqian`；后端迁移在 `peiqian` 上用 `cargo run --bin run_migrations` 应用（`_sqlx_migrations` 表记账），部署脚本不会自动跑迁移
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

**生产部署脚本**：
- `./deploy_jd_docker.sh` — Docker 方式（out109 build → Harbor → jd 拉镜像）
- `./deploy_peiqian_docker.sh` — Docker 方式（peiqian 本机 build + 本地镜像直接运行，不推 Harbor；自动清理只保留最近 10 个镜像；git 走 peiqian clash 代理）
- `bash deploy/deploy-backend-docker.sh` — 旧 Docker 脚本，默认部署到 jd
- `bash deploy/deploy-backend-binary.sh` — systemd 方式（jd 上 cargo build --release → systemctl restart）
- 生产默认用 Docker 方式，systemd 为备用

### 前端

```bash
cd football_insight_mini
bun install
bun run dev:h5
bun run build:h5
bun run dev:mp-weixin
bun run build:mp-weixin
```

**小程序发版与审核版本号**：发版用 `bun run mp:release`（构建 + 向本项目后端 mini-review 登记库申请版本号 + `miniprogram-ci` 上传）。**发版在 peiqian 上执行（推荐）**：上传私钥（`football_insight_mini/private.<appid>.key`）、`.env.ci.local`、bun 都已就位，且从干净 checkout 构建保证上传内容与 main 完全一致（本地 Mac 也能发，但必须先把改动全部提交推送）。审核版本号由后端 `f_i_mini_review_statuses` 登记库统一分配，不要手动改 `manifest.json` 的 `versionName`；指定版本用 `MINI_PROGRAM_VERSION=x.y.z`，离线构建用 `MINI_REVIEW_SKIP=1`。完整步骤与 ssh 命令见 `football_insight_mini/README.md` 的「微信小程序发布」一节。

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
