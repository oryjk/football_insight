# Football Insight Deployment

这份文档描述当前 Football Insight 在生产环境的部署约定。

## 生产环境

- 线上域名入口机：`jd`（`match.oryjk.cn` 的 DNS 解析到 jd 公网 IP `117.72.164.211`）
- jd 登录：`ssh jd`；jd 上的 Nginx 把 `/api/v1/` 和 `/football/wechat/webhook` 反代到 `peiqian` 的 `10.8.10.2:8092`（WireGuard），`/football/` 前端静态资源由 jd 本地直接服务
- 真正承接线上 API/webhook 流量的后端：`peiqian`（`ssh peiqian`）上的 Docker 容器；发版 peiqian 即对线上生效
- jd 上也保留一个后端容器，但不在流量路径上（备用，无需每次同步发版）
- 前端访问地址：`https://match.oryjk.cn/football/`
- 后端接口前缀：`https://match.oryjk.cn/api/v1/`
- 结论：后端发版发 `peiqian` 即生效；`jd` 容器为备用，无需每次同步发版

## 生产目录

- monorepo 目录：`/root/projects/football_insight`
- 后端项目目录：`/root/projects/football_insight/football_insight_service_backend_rs`
- `peiqian` 运行时 env 文件：`/root/projects/football_insight/football-insight-service-backend-rs.env`
- scraper 生产运行机器：`local233`
- scraper monorepo 目录：`/home/betalpha/projects/football_insight`
- scraper 项目目录：`/home/betalpha/projects/football_insight/sina_csl_scraper`
- 后端应用日志：`/root/projects/football_insight/football_insight_service_backend_rs/logs/app.log`
- 后端滚动日志：`/root/projects/football_insight/football_insight_service_backend_rs/logs/app.<timestamp>.log`
- 后端生产容器：`football-insight-service-backend-rs`
- systemd 备用服务：`football-insight.service`
- 前端静态目录：`/root/docker_data/nginx/html/football/`

## Nginx 约定

- `jd` 是当前线上域名入口机
- `peiqian` 当前已确认本机后端 `8092` 和 `ticket-monitor-axum` `4000` 正常监听
- 当前只允许修改 football 相关路径
- 如果发现路由冲突，不要直接修改其他服务配置，先确认
- 如果修改了 Nginx 配置：
  - 先做备份
  - 先跑 `nginx -t`
  - 不要 `reload`
  - 直接重启 Nginx Docker 容器

### 更换 `match.oryjk.cn` TLS 证书

在本地 Mac 执行：

```bash
bash deploy/renew-jd-nginx-ssl.sh [证书目录]
```

- 证书目录需包含 `match.oryjk.cn.pem`（含中间证书的 fullchain）和 `match.oryjk.cn.key`；不传参数时自动选取 `~/Downloads` 下最新的 `*_match.oryjk.cn_nginx` 目录
- 流程：本地校验（文件存在 / 域名匹配 / 有效期 / 证书私钥配对）-> 远端备份旧证书到 `ssl.bak.<时间戳>` -> 覆盖上传 -> `nginx -t`（失败自动回滚）-> `nginx -s reload` -> 回读线上证书确认生效

## 前端部署

### 本地构建

```bash
cd football_insight_mini
bun install
bun run build:h5
```

### 同步到生产

```bash
rsync -av --delete \
  football_insight_mini/dist/build/h5/ \
  jd:/root/docker_data/nginx/html/football/
```

### 前端上线后验证

```bash
curl -I https://match.oryjk.cn/football/
```

浏览器里至少检查：

- 首页是否能打开
- 榜单页是否能正常拉数据
- 我的页面是否能正常读取系统配置

### Web 管理后台静态构建

`football_insight_admin_web` 独立产出静态文件，不包含服务端。

```bash
cd football_insight_admin_web
bun install --frozen-lockfile
bun run test
bun run build
```

输出目录为 `football_insight_admin_web/dist/`。根路径开发使用同源 `/api/v1/admin/**`；
子路径部署使用 `<部署路径>/api/v1/admin/**`，Nginx 移除部署路径后转到足球后端。
需要先部署包含商品 CRUD 的后端，再发布前端。子路径构建可用
`ADMIN_WEB_BASE_PATH=/football-admin/ bun run build`，路径必须与实际静态托管配置一致。
local233 内网入口已发布为 `http://172.16.60.233/football-admin/`，前端共享静态根目录为
`/home/betalpha/.local/share/betalpha-admin-downloads`，`football-admin` 链接指向独立版本目录。
发布命令：`bash deploy/deploy-admin-web-local233.sh`。API 仅在 `/football-admin/api/v1/admin/`
范围反代到本机 Cargo 后端 18092，保留网关现有 Mesh `/api/v1/` 和其他服务路由。
该后端连接现有生产库，商品变更会直接落库；本次没有更新 peiqian 生产后端。
Nginx 配置文件位于 `/home/betalpha/services/betalpha-internal-gateway/nginx.conf`；
路由片段见 `deploy/nginx/local233-admin-web.conf`。变更前备份，`nginx -t` 通过后重启网关容器。
不要把后台构建输出覆盖现有小程序 H5 的 `/football/`。

管理员初始化不需要默认密码：未配置 `ADMIN_OWNER_PASSWORD` 且管理员表为空时，后台页面会要求为固定账号 `superadmin` 设置密码；已有管理员保持原账号和密码。登录后在「设置」修改当前账号或密码，保存后全部会话退出。共享同一数据库的所有后端都要升级账号初始化逻辑，之后才开放账号改名；旧版本 bootstrap 可能在重启时重建配置中的旧账号。不要在部署日志或文档中记录密码。
新增 Nginx 路由时继续遵守本文件的 football 路径和冲突处理约定。

## 后端部署

当前后端同时保留两种部署方式：

- 首选：Docker 镜像部署，运行容器 `football-insight-service-backend-rs`
- 备用：systemd 直接运行 release binary，服务名 `football-insight.service`

常规发布优先使用 Docker。systemd unit 模板保留在 `football_insight_service_backend_rs/deploy/football-insight.service`，仅在明确需要备用部署时使用。

### 1. 推送代码

先保证本地代码已经提交并 push。

### 2. Docker 发布

部署到 `jd`：

```bash
cd football_insight_service_backend_rs
./deploy_jd_docker.sh
```

部署到 `peiqian`：

```bash
cd football_insight_service_backend_rs
./deploy_peiqian_docker.sh
```

两个 Docker 发布脚本都会先检查本地提交已经 push 到 `origin/main`。

其中：

- `deploy_jd_docker.sh`：在 `out109` 构建镜像并推送到 Harbor，`jd` 拉取镜像并重启容器（可用 `BUILD_HOST` / `BUILD_REPO_DIR` 环境变量换构建机）；发布前需要在本地 `football_insight_service_backend_rs/.env` 或环境变量中提供 Harbor 凭据 `HARBOR_PASSWORD`，不要提交真实 `.env`
- `deploy_peiqian_docker.sh`：在 `peiqian` 本机构建。**不要直接用它**：peiqian 拉基础镜像极慢（实测约 200KB/秒，会卡死）。生产发布改为在 local233 构建镜像、传到 peiqian 再启动，步骤见下面「Docker 发布（local233 构建）」

### Docker 发布（local233 构建，生产默认）

```bash
cd football_insight_service_backend_rs
TAG=$(git rev-parse --short HEAD)
docker build -t football-insight-service-backend-rs:$TAG -t football-insight-service-backend-rs:latest .
docker save football-insight-service-backend-rs:$TAG football-insight-service-backend-rs:latest | gzip | ssh peiqian 'gunzip | docker load'
ssh peiqian "docker rm -f football-insight-service-backend-rs; docker run -d --name football-insight-service-backend-rs --restart unless-stopped --network host --env-file /root/projects/football_insight/football-insight-service-backend-rs.env -e PORT=8092 -v /root/projects/football_insight/football_insight_service_backend_rs/logs:/app/logs football-insight-service-backend-rs:$TAG"
```

- 先确认本地提交已 push；发布后检查 `curl http://127.0.0.1:8092/api/health` 和容器 `healthy`
- 回滚：用上一个镜像 tag 重新执行最后一条 `docker run`（peiqian 保留历史 tag，可用 `docker images football-insight-service-backend-rs` 查看）
- 环境文件 `football-insight-service-backend-rs.env` 不在 Git 里；修改前先备份（例如 `cp -p x.env x.env.before-<说明>-<日期>`）

### 3. 数据库迁移

生产数据库 `football-data-postgres` 运行在 `peiqian`，不对外开放公网端口。如果这次改动包含 migration，用**本次要发布的新镜像**里的 `/usr/local/bin/run_migrations` 在 peiqian 上执行（`_sqlx_migrations` 表记账，部署脚本不会自动跑迁移）。`sqlx::migrate!` 在编译时把 `migrations/` 嵌进二进制，所以 peiqian 不需要源码、不需要 `git pull`、不需要 cargo：

```bash
# 镜像 docker load 到 peiqian 之后、切换容器之前执行（迁移先于新后端发布）
ssh peiqian "docker run --rm --network host \
  --env-file /root/projects/football_insight/football-insight-service-backend-rs.env \
  --entrypoint /usr/local/bin/run_migrations \
  football-insight-service-backend-rs:$TAG"
```

- 必须用包含新迁移文件的镜像 tag；旧镜像里没有新迁移，会直接输出成功但什么也不做
- 执行前后核对 `_sqlx_migrations`，确认只应用了本次新增的版本
- 临时执行单个 SQL 文件：镜像内 `/app/migrations/` 也带有迁移文件，在上面命令末尾追加 `/app/migrations/<your_migration>.sql`；镜像外的文件用 `-v` 挂载进去再传路径

不再在 peiqian 源码目录用 `cargo run --bin run_migrations`，也不再把“服务器全局安装 `sqlx`”作为前提。

### 4. systemd 备用发布

只有明确不用 Docker 时，才走 systemd 备用流程。下面示例是 `jd`：

```bash
ssh jd
cd /root/projects/football_insight
git pull --ff-only
cd /root/projects/football_insight/football_insight_service_backend_rs
cargo build --release
systemctl restart football-insight.service
systemctl status football-insight.service --no-pager
```

不要用 `nohup`、`service.pid`、裸 `cargo run` 或依赖 SSH 会话的后台进程托管生产后端。只有修改 systemd unit 文件后才需要执行 `systemctl daemon-reload`。

### 5. 后端上线后验证

部署到 `jd`：

```bash
ssh jd 'docker ps --filter name=football-insight-service-backend-rs'
ssh jd 'curl http://127.0.0.1:8092/api/health'
```

部署到 `peiqian`：

```bash
ssh peiqian 'docker ps --filter name=football-insight-service-backend-rs'
ssh peiqian 'curl http://127.0.0.1:8092/api/health'
```

线上入口验证：

```bash
curl https://match.oryjk.cn/api/v1/live/overview
curl https://match.oryjk.cn/api/v1/system/public-config
```

如果本次涉及认证，再额外检查：

- 注册
- 登录
- `/api/v1/auth/me`

如果本次涉及公众号 webhook，再额外检查：

- `GET /football/wechat/webhook`
- 关注事件或文本指令实际回调

## 最佳盯盘时段（离线统计）部署

功能：当前比赛页的「最佳盯盘时段」面板。数据由 ticket-monitor 的离线批处理按历史主场比赛统计，**写入 football_data 的 `f_i_ticket_restock_profiles`（每场一行，按 match_id 幂等 upsert）**；football 后端每 10 分钟后台聚合，优先读这张表，库里没有或不可用时才回退到下面的 4001 读接口，前台只读内存缓存。设计见 `docs/superpowers/specs/2026-10-16-best-watch-windows-design.md`。

全部运行在 `peiqian`。peiqian 上的 monitor 抢票容器（`ticket-monitor-axum`，镜像停在旧版本）**不动**，统计功能走独立镜像，避免把 monitor master 上未上线的改动带到抢票服务。

| 组件 | 位置 | 说明 |
|---|---|---|
| 镜像 `ticket-restock-profiles` | peiqian 本地 | 由 monitor 仓库 `Dockerfile.restock` 构建，只含 `rebuild-restock-profiles` 和 `restock-profile-server`，不含抢票程序 |
| 快照读取容器 `ticket-restock-profile-server` | host 网络，`127.0.0.1:4001` | 只读挂载 `/root/projects/ticket-monitor-axum/data`，快照在 `data/restock-profiles/<match_id>.json`；不连数据库 |
| 批处理 | systemd `ticket-restock-profiles.service` + `.timer`（`/etc/systemd/system/`） | `*-*-1/2 04:00:00 Asia/Shanghai`，即每两天北京时间凌晨 4 点；peiqian 系统时区是 UTC，所以必须带 `Asia/Shanghai`。已有快照的比赛自动跳过，`--force` 才强制重算 |
| 批处理凭据 | `/root/projects/ticket-monitor-axum/restock.env`（0600） | `RS_DATABASE_URL`（监控 MySQL，批处理会话强制 `READ ONLY`，单条查询上限 60 秒）+ `FOOTBALL_DATABASE_URL`（football_data，复用后端账号，用于写快照表；不设则退回只写文件）。批处理写库成功后仍镜像一份 JSON，供 4001 读接口过渡期使用 |
| football 配置 | `football-insight-service-backend-rs.env` 里的 `TICKET_RESTOCK_PROFILE_BASE_URL=http://127.0.0.1:4001` | 只用于读快照；未设置时回退 `TICKET_MONITOR_BASE_URL` |

统计口径：只统计已结束的成都蓉城主场、开售后 10 分钟起算、按北京时间小时分桶；批处理遇到正在售票或 2 分钟内开售的比赛会整轮跳过，不影响抢票。非 V6 用户的预览响应里不含具体小时（`start_hour`/`end_hour` 为 null）。

### 发布或更新

1. 在 local233 的 monitor 仓库构建并传输镜像（不要在 peiqian 上构建）：

```bash
cd /home/betalpha/projects/ticket/ticket-monitor-axum
docker build -f Dockerfile.restock -t ticket-restock-profiles:$(git rev-parse --short HEAD) .
docker tag ticket-restock-profiles:$(git rev-parse --short HEAD) ticket-restock-profiles:current
docker save ticket-restock-profiles:$(git rev-parse --short HEAD) ticket-restock-profiles:current | gzip | ssh peiqian 'gunzip | docker load'
```

2. 重启读取容器：`ssh peiqian 'docker rm -f ticket-restock-profile-server; docker run -d --name ticket-restock-profile-server --restart unless-stopped --network host -v /root/projects/ticket-monitor-axum/data:/app/data:ro ticket-restock-profiles:current'`
3. 如 unit 文件有变化，复制 `deploy/systemd/ticket-restock-profiles.{service,timer}` 到 `/etc/systemd/system/` 并 `systemctl daemon-reload`。
4. football 后端按上面「Docker 发布（local233 构建）」发布。

### 验证

```bash
# 手动跑一遍批处理（已有快照应全部 unchanged）
ssh peiqian 'systemctl start ticket-restock-profiles.service; journalctl -u ticket-restock-profiles.service --since "-3min" --no-pager | grep "restock offline batch"'
# 下次定时触发时间
ssh peiqian 'systemctl list-timers ticket-restock-profiles.timer --no-pager'
# 读接口（since 是该场开售后 10 分钟，带偏移）
ssh peiqian 'curl -s "http://127.0.0.1:4001/api/match/restock-profile/583?since=2026-10-08T14:10:00%2B08:00"'
# football 公开接口：匿名应为 preview，且 start_hour/end_hour 为 null
ssh peiqian 'curl -s http://127.0.0.1:8092/api/v1/ticket-watch/best-windows'
```

### 回滚与排查

- 关闭功能：`systemctl disable --now ticket-restock-profiles.timer` 并 `docker stop ticket-restock-profile-server`；football 后端刷新失败时会保留上一次成功的聚合结果继续展示，重启 football 容器后（缓存清空且读不到快照）面板才会静默隐藏，其他功能不受影响。
- 本地后端（local233，连生产 `football_data`）不再需要到 peiqian 的隧道：快照在库里，直接读表。
- 面板消失：先看 `best-windows` 接口的 `status`（`ok` / `insufficient` / `unavailable`），再看 `f_i_ticket_restock_profiles` 是否有最新比赛的行（或 4001 读接口 / `data/restock-profiles`）；最近 8 场里没就绪的比赛会被跳过，不影响其余样本。
- 批处理失败：单场失败只记录并继续，整轮结束时才以非零退出；`journalctl -u ticket-restock-profiles.service` 看是哪场、什么原因。

## 球队名单同步（战术板）

战术板（最佳阵容）用的完整名单来自新浪 `/op/api/team/player`，由 local233 的抓取器写入生产 `f_i_squad_players`，后端 `GET /api/v1/lineups/squads/{team_id}` 只读。

- 迁移：`20261019120000_add_squad_players.sql`（2026-10-19 已在 peiqian 用 `cargo run --release --bin run_migrations` 执行；peiqian 后端目录没有 `.env`，需先 `set -a; . ../football-insight-service-backend-rs.env; set +a` 加载 `DATABASE_URL`）。
- 首次回填（往前 3 个赛季的前队员）：`cd sina_csl_scraper && FI_SQUAD_SYNC_BACKFILL=3 ./sync_squads_daily.sh`。头像逐张转存 MinIO，16 队首次超过半小时；同步是幂等的，被超时打断后重跑即可补齐（2026-10-19 首次因 25 分钟超时只跑完 14 队，重跑补齐 16 队）。
- 每日调度：`sina_csl_scraper/deploy/squad-sync.cron.example`，每天 03:45 只同步当前赛季（`FI_SQUAD_SYNC_BACKFILL` 默认 0），日志 `sina_csl_scraper/squad_sync.log`。
- 失败隔离：每队独立；某队请求失败、结构异常或返回空名单时跳过该队，不改它的任何数据，摘要 JSON 的 `teams_failed` 列出失败球队。只有当前赛季成功拿到非空名单，才把缺席者置为前队员（`active=false`，不删除）。
- 只读核对：`curl -s http://127.0.0.1:18092/api/v1/lineups/squads/77680`，看 `active` / `former` 人数；接口返回 503 表示表不存在或库不可用。

### 球员头像小图补生成（一次性 + 幂等）

球员头像的 96/240px 小图（`summary/players/thumbs/<尺寸>/<文件名>.jpg`）由抓取器在上传新头像时顺带生成；存量头像用补生成命令（会写生产 MinIO，在 local233 的 `sina_csl_scraper` 目录执行）：

1. 先演练：`uv run sina-csl-scraper sync-avatar-thumbs --dry-run`（只读库统计候选数，不下载、不上传；需要 `FI_DATABASE_URL` 或 `DATABASE_URL`）。
2. 确认统计合理后正式执行：`uv run sina-csl-scraper sync-avatar-thumbs`（需要 MinIO 凭据环境变量，与 `sync-squads` 相同）。

命令幂等（已存在的小图用 `head_object` 跳过），可重复执行；`--limit N` 限制处理张数；单张失败继续其余，最后输出 JSON 统计（`candidates`/`skipped`/`generated`/`failed`/`ignored`），有失败时退出码非 0。补生成前小程序会自动回退原图。

## 战术板阵容（`f_i_lineups`）

- 迁移：`20261019130000_add_lineups.sql`（2026-10-19 已在 peiqian 执行；加载 env 文件后 `cargo run --release --bin run_migrations`，注意 peiqian 后端目录没有 `.env`）。迁移必须先于新后端发布。
- 接口：`GET/PUT /api/v1/lineups/me/{team_id}` 需要登录；`GET /api/v1/lineups/squads/{team_id}` 公开。
- 部署：沿用 local233 构建镜像 → `docker save | gzip | ssh peiqian 'gunzip | docker load'` → 用现有运行参数重启容器；回滚用上一个镜像 tag（当前 `97b025f`）。
- 核对：只读 `curl https://match.oryjk.cn/api/v1/lineups/squads/77680`；`/lineups/me/…` 未登录应为 401。验证写入需要登录令牌，不要用真实用户令牌。
- 球迷共识阵容（2026-10-20 已上线：生产后端 `3925206`，回滚 `82e643f`；1.1.0 robot 2 已上传）：**无数据库迁移**，上线只需 ① 部署并重启生产后端 → ② 上传小程序体验版。部署后核对 `curl -i https://match.oryjk.cn/api/v1/lineups/teams/77680/consensus` 匿名应为 200（未达 10 套时 `unlocked=false` 只返回进度）。

## 转会投票（`f_i_transfer_*`）

- 迁移：`20261020120000_add_transfer_votes.sql`，创建窗口、提名卡、投票三张表，并写入第一期“2026 冬窗”。**2026-10-20 已在 peiqian 执行并登记**；生产后端已部署为 `82e643f`（回滚镜像 `cd3fcc4`），小程序 1.1.0 robot 2 已上传。
- 上线顺序（三步都要做，顺序不能换）：① 在 peiqian 用 `cargo run --bin run_migrations` 应用 `20261020120000`（peiqian 后端目录没有 `.env`，先 `set -a; . ../football-insight-service-backend-rs.env; set +a`）→ ② 部署并重启生产后端 → ③ 上传小程序体验版。
- 迁移应用前，已含新代码的后端上 `/api/v1/transfers/**` 会返回 503（表不存在），这是预期状态。
- 新开一期：往 `f_i_transfer_windows` 插入一行即可，无需改代码。示例（2027 夏窗）：

```sql
INSERT INTO f_i_transfer_windows (code, name, opens_at, closes_at)
VALUES ('2027-summer', '2027 夏窗', '2027-06-01T00:00:00+08:00', '2027-08-01T00:00:00+08:00');
```

- 核对：迁移并部署后 `curl https://match.oryjk.cn/api/v1/transfers/windows/current` 应返回当期窗口 JSON；榜单接口 `GET /api/v1/transfers/teams/{team_id}/board` 公开，提名与投票接口需要登录。
- 第二期（拉票海报与小程序码，2026-10-20 已上线：生产后端 `3925206`，1.1.0 robot 2 已上传）：**无数据库迁移**，上线只需 ① 部署并重启生产后端（小程序码用已有的小程序 appid/secret 环境变量，与战术板小程序码共用同一对凭据）→ ② 上传小程序体验版。部署后核对 `curl -i https://match.oryjk.cn/api/v1/transfers/nominations/XXXXXXXX/wxacode` 未登录应为 401。

## 日志查看

应用代码会把 tracing 日志写入 `logs/` 目录，并按 10MB 滚动；这是排查业务日志时的优先入口（线上流量在 `peiqian`，排查线上问题优先看 peiqian；jd 备用容器同理把 `ssh peiqian` 换成 `ssh jd`）：

```bash
ssh peiqian 'tail -n 100 -f /root/projects/football_insight/football_insight_service_backend_rs/logs/app.log'
```

Docker 容器 stdout/stderr 作为进程级兜底日志；应用业务日志仍优先看 `logs/app.log`：

```bash
ssh peiqian 'docker logs --tail 100 football-insight-service-backend-rs'
```

如果切换到 systemd 备用部署，再用 journal 排查进程启动失败、panic 或早期 stderr 输出：

```bash
ssh peiqian 'journalctl -u football-insight.service -n 100 --no-pager'
```

说明：

- `logs/app.log` 由后端代码直接写入，不保留 ANSI 颜色
- `logs/app.log` 达到 10MB 后会滚动为 `logs/app.<timestamp>.log`
- 当前代码最多保留 100 个滚动文件
- Docker 部署下，容器挂载 `logs/`，业务日志仍写入 `logs/app.log`
- systemd 备用部署下，unit 使用 `StandardOutput=null` 和 `StandardError=journal`
- 历史遗留的 `service.log` 不再继续增长，不再作为当前日志入口

## 当前发布顺序建议

### 纯前端改动

1. 本地 `bun run build`
2. 同步 `dist/`
3. 浏览器验证页面

### 纯后端改动

1. push 代码
2. 发 `peiqian` 即对线上生效（线上流量由 peiqian 后端容器承接）：
   - `football_insight_service_backend_rs/deploy_peiqian_docker.sh`
   - `jd` 上的容器是备用、不在流量路径上，无需每次同步发版；确需同步时再跑 `football_insight_service_backend_rs/deploy_jd_docker.sh`
3. 如有 migration，在切换容器前用新镜像里的 `run_migrations` 执行（见上文「3. 数据库迁移」）
4. 用 `https://match.oryjk.cn` 域名验证接口（目标机本机 8092 正常不代表线上生效）
5. 优先看 `logs/app.log`，必要时再看 `docker logs`

### 同时改前后端

1. 先发布后端
2. 验证接口正常
3. 再同步前端静态文件
4. 最后整站联调

### scraper 改动

1. push 代码
2. `local233` 上 `cd /home/betalpha/projects/football_insight && git pull --ff-only`
3. `cd sina_csl_scraper`
4. 如依赖有变化，执行 `uv sync`，必要时执行 `uv run playwright install chromium`
5. 保留 `.env.sync`、`.auto_sync_state.json`、`auto_sync.log`，不要提交这些运行文件
6. crontab 模板见 `sina_csl_scraper/deploy/auto-sync.cron.example`
7. 每日头像同步模板见 `sina_csl_scraper/deploy/avatar-sync.cron.example`
8. 查看赛后同步日志：`tail -n 100 -f /home/betalpha/projects/football_insight/sina_csl_scraper/auto_sync.log`
9. 查看头像同步日志：`tail -n 100 -f /home/betalpha/projects/football_insight/sina_csl_scraper/avatar_sync.log`

### 小程序改动

默认在 local233 的 `/home/betalpha/projects/football_insight/football_insight_mini` 从干净 main 执行 `bun run mp:release -- --robot 2 --desc "本次变更说明"`，统一完成生产构建、审核版本登记和测试上传。local233 已配置上传私钥和 `.env.ci.local`（0600 权限，Git 忽略），登记地址为 `https://match.oryjk.cn/api/v1`；开发、构建与上传在同一台 Ubuntu 机器完成，无需连接 Mac。上传前先提交推送全部业务改动；版本文件若因登记发生变化，也需提交推送后重跑。

local233 无法发布时可使用 peiqian 已配置私钥、`.env.ci.local` 与 Bun 的备用环境；先通过既有 clash 代理更新干净 main，再执行同一发布入口。robot 2 上传不代替微信公众平台提审和正式发布。

审核版本号由后端 `f_i_mini_review_statuses` 统一分配，不手动修改 manifest。指定版本用 `MINI_PROGRAM_VERSION=x.y.z`，离线构建用 `MINI_REVIEW_SKIP=1`；正式上传不会跳过登记。完整步骤见 `football_insight_mini/README.md` 的「微信小程序发布」一节。

## local233 联调后端部署

根目录 `deploy_local233.sh` 把已提交并 push 的代码部署到 local233 的 Cargo 联调后端（统一端口 `18092`）：

```bash
./deploy_local233.sh
```

- 要求本地和远端工作区都干净；从开发机执行时本地 HEAD 必须已 push 到 `origin`
- 也可以直接登录 local233，在 `/home/betalpha/projects/football_insight` 里执行同一命令（脚本检测到自身位于部署工作区后进入本机模式，直接从 origin 拉当前分支）
- 内部调用 `deploy/local-test/up.sh` 编译并重启用户级 `football-insight-local-backend.service`，该服务执行 `cargo run --bin football_insight_service_backend_rs`；完成后校验服务与 `/api/health` 等接口，启动不运行迁移
- 可用 `DEPLOY_HOST` / `DEPLOY_REPO_DIR` / `DEPLOY_BRANCH` 环境变量覆盖目标；端口固定 18092，不再部署本地后端容器
- 详细说明见 `deploy/local-test/README.md`

## local233 生产库 SSH 隧道

local233 管理联调后端经 SSH 隧道连接 peiqian 上的生产 `football_data`：

- 隧道由 systemd 用户服务 `football-insight-db-tunnel.service` 常驻托管（unit 位于 `~/.config/systemd/user/`，运行用户 `betalpha`，已 enable + linger）
- `Restart=always` 自愈，机器重启后自动拉起，不再依赖 `up.sh` 手工建隧道
- 服务保留 master-socket（`/tmp/football-insight-admin-db-tunnel-1000.sock`）；`deploy/local-test/up.sh` 通过 `systemctl --user start football-insight-db-tunnel.service` 确保隧道已启动，开发环境停止时保留该共享隧道
- 排查命令：`systemctl --user status football-insight-db-tunnel`（在 local233 上以 betalpha 执行）

## 风险点

- 线上域名入口是 `jd`（DNS 解析到 jd），但 jd 的 Nginx 把 `/api/v1/` 和 `/football/wechat/webhook` 反代到 `peiqian`；只把镜像发到 `jd` 线上不会有任何变化，发 `peiqian` 才生效
- 后端部署前如果忘了 push，服务器拉不到最新代码
- 有 migration 时，如果只重启服务不跑迁移，接口可能变成 `500`
- 如果后端代码改了，本地验证前必须先重启本地后端，不要在旧进程上验证新逻辑

## 战术板分享、点赞与公开榜单（2026-10-19）

- `20261019140000_add_lineup_likes.sql` 已在 peiqian 用 `cargo run --bin run_migrations` 无参数执行并登记；新增 `f_i_lineup_likes` 与查询索引。
- 满 11 人保存即公开，不足员隐藏。公开读：`GET /api/v1/lineups/teams/{team_id}?sort=latest|likes&page=1&page_size=20`、`GET /api/v1/lineups/shared/{code}`；点赞 `PUT/DELETE .../like` 与二维码 `GET .../wxacode` 需要活跃账号。
- 上线镜像 `b131306` 在 local233 构建，传输到 peiqian 后启动，healthy、重启次数 0。保留旧镜像 `68c19f3` 和容器 `football-insight-service-backend-rs-rollback-68c19f3-20261019-community`。回滚旧容器不需要删除新增表。
- 小程序 1.1.0 robot 2 已从干净 main `b131306` 上传；完整验证见 `docs/lineup-sharing-community-validation.md`。
