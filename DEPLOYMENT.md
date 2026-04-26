# Football Insight Deployment

这份文档描述当前 Football Insight 在生产环境的部署约定。

## 生产环境

- 服务器别名：`jd`
- 服务器登录：`ssh jd`
- 前端访问地址：`https://match.oryjk.cn/football/`
- 后端接口前缀：`https://match.oryjk.cn/api/v1/`

## 生产目录

- monorepo 目录：`/root/projects/football_insight`
- 后端项目目录：`/root/projects/football_insight/football_insight_service_backend_rs`
- scraper 生产运行机器：`local233`
- scraper monorepo 目录：`/home/betalpha/projects/football_insight`
- scraper 项目目录：`/home/betalpha/projects/football_insight/sina_csl_scraper`
- 后端应用日志：`/root/projects/football_insight/football_insight_service_backend_rs/logs/app.log`
- 后端滚动日志：`/root/projects/football_insight/football_insight_service_backend_rs/logs/app.<timestamp>.log`
- 后端生产容器：`football-insight-service-backend-rs`
- systemd 备用服务：`football-insight.service`
- 前端静态目录：`/root/docker_data/nginx/html/football/`

## Nginx 约定

- `jd` 上 Nginx 运行在 Docker 容器里
- 当前只允许修改 football 相关路径
- 如果发现路由冲突，不要直接修改其他服务配置，先确认
- 如果修改了 Nginx 配置：
  - 先做备份
  - 先跑 `nginx -t`
  - 不要 `reload`
  - 直接重启 Nginx Docker 容器

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

## 后端部署

当前后端同时保留两种部署方式：

- 首选：Docker 镜像部署，运行容器 `football-insight-service-backend-rs`
- 备用：systemd 直接运行 release binary，服务名 `football-insight.service`

常规发布优先使用 Docker。systemd unit 模板保留在 `football_insight_service_backend_rs/deploy/football-insight.service`，仅在明确需要备用部署时使用。

### 1. 推送代码

先保证本地代码已经提交并 push。

### 2. Docker 发布

```bash
cd football_insight_service_backend_rs
./deploy_jd_docker.sh
```

Docker 发布脚本会完成：

- 检查本地提交已经 push 到 `origin/main`
- 在 `out109` 拉取最新 monorepo
- 在 `out109` 构建 Docker 镜像并推送到 Harbor
- 在 `jd` 拉取镜像
- 停用旧 `football-insight.service`
- 重建并启动 `football-insight-service-backend-rs` 容器
- 挂载 `/root/projects/football_insight/football_insight_service_backend_rs/logs` 到容器 `/app/logs`
- 验证 `http://127.0.0.1:8092/api/health`

发布前需要在本地 `football_insight_service_backend_rs/.env` 或环境变量中提供 Harbor 凭据，例如 `HARBOR_PASSWORD`。不要提交真实 `.env`。

### 3. 数据库迁移

如果这次改动包含 migration，优先在生产目录执行仓库内置迁移入口：

```bash
ssh jd
cd /root/projects/football_insight/football_insight_service_backend_rs
cargo run --release --bin run_migrations
```

如果需要临时执行某一条单独的 SQL 文件，也可以直接走项目内置入口：

```bash
cd /root/projects/football_insight/football_insight_service_backend_rs
cargo run --release --bin run_migrations -- migrations/<your_migration>.sql
```

当前不再把“服务器全局安装 `sqlx`”作为默认前提，优先使用仓库内的 `run_migrations`。

### 4. systemd 备用发布

只有明确不用 Docker 时，才走 systemd 备用流程：

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

```bash
ssh jd 'docker ps --filter name=football-insight-service-backend-rs'
ssh jd 'curl http://127.0.0.1:8092/api/health'
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

## 日志查看

应用代码会把 tracing 日志写入 `logs/` 目录，并按 10MB 滚动；这是排查业务日志时的优先入口：

```bash
ssh jd 'tail -n 100 -f /root/projects/football_insight/football_insight_service_backend_rs/logs/app.log'
```

Docker 容器 stdout/stderr 作为进程级兜底日志；应用业务日志仍优先看 `logs/app.log`：

```bash
ssh jd 'docker logs --tail 100 football-insight-service-backend-rs'
```

如果切换到 systemd 备用部署，再用 journal 排查进程启动失败、panic 或早期 stderr 输出：

```bash
ssh jd 'journalctl -u football-insight.service -n 100 --no-pager'
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
2. 优先执行 `football_insight_service_backend_rs/deploy_jd_docker.sh`
3. 如有 migration，再在 `jd` 执行 `cargo run --release --bin run_migrations`
4. `curl` 验证接口
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

小程序不通过 `jd` 上的 Nginx 发布，发布流程是：

1. 本地 `bun run build:mp-weixin`
2. 用微信开发者工具打开 `dist/build/mp-weixin`
3. 在微信开发者工具里上传新版本

## 风险点

- `jd` 上 Nginx 承载了其他服务，不要顺手改非 football 路由
- 后端部署前如果忘了 push，服务器拉不到最新代码
- 有 migration 时，如果只重启服务不跑迁移，接口可能变成 `500`
- 如果后端代码改了，本地验证前必须先重启本地后端，不要在旧进程上验证新逻辑
