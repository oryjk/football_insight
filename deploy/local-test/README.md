# Local233 Cargo Development Environment

local233 后端统一直接通过 `cargo run --bin football_insight_service_backend_rs` 启动，监听 `18092`。小程序、独立 H5、Web 后台均使用 `http://172.16.60.233:18092`；线上 peiqian 后端继续使用 Docker。

- 后端：用户级 `football-insight-local-backend.service` 托管 `cargo run`，支持重启与开机恢复。
- Redis：Docker 容器 `football-insight-local-test-redis`，主机端口 `127.0.0.1:56379`。
- 数据库：经 `football-insight-db-tunnel.service` 访问 `127.0.0.1:55433`，连接生产 `football_data`。真实商品、会员与用户写操作会直接生效。
- 开发启动不自动运行数据库迁移；迁移按 [生产部署说明](../../DEPLOYMENT.md) 单独执行。

## 本地运行配置

后端项目目录下的 `.env` 被 Git 忽略，权限必须为 `0600`。现有 local233 配置已从原本地容器安全迁移，保留 JWT、管理员及微信小程序登录配置；不要将配置值写进提交或日志。

首次配置其它开发目录时，复制 `football_insight_service_backend_rs/.env.example` 为 `.env`，填写实际配置，并设置：

```dotenv
PORT=18092
# DATABASE_URL 使用本机 127.0.0.1:55433 的私有隧道
REDIS_URL=redis://127.0.0.1:56379
TICKET_MONITOR_BASE_URL=http://127.0.0.1:4000
PUBLIC_BASE_URL=http://172.16.60.233:18092
FI_REFLUX_NOTIFICATION_WORKER_ENABLED=false
```

## 后台启动或重启

仓库根目录执行：

```bash
./deploy/local-test/up.sh
journalctl --user -u football-insight-local-backend.service -f
```

脚本先编译，再启动数据库隧道、Redis，并重启 Cargo 后端。旧本地 Docker 后端会停止并取消自动重启，不再构建后端镜像。编译失败不会中断当前后端。

停止后端与 Redis：

```bash
./deploy/local-test/down.sh
```

共享数据库隧道由独立服务管理，停止开发环境不会关闭隧道。

## 前台直接运行

依赖服务启动后，可停止后台实例，再在后端目录直接运行（读取同一份 `.env`）：

```bash
systemctl --user stop football-insight-local-backend.service
cd football_insight_service_backend_rs
cargo run --bin football_insight_service_backend_rs
```

修改 Rust 代码后重启该进程；不要在旧实例上验证。结束前台调试后，回仓库根目录执行 `./deploy/local-test/up.sh` 恢复常驻实例。

## 从其它开发机更新 local233

```bash
./deploy_local233.sh
```

该脚本要求本地和远端工作区干净，当前业务提交已 push；在 local233 本机执行时直接拉取当前分支。可设置 `DEPLOY_HOST`、`DEPLOY_REPO_DIR`、`DEPLOY_BRANCH`。更新后调用上述 Cargo 启动脚本，并核对用户服务和 API；端口固定为 18092。

## 云端开发自动同步与上传守护服务（local233）

当在远程云主机修改代码并 `git push` 到 `origin/main` 时，可在 `local233` 启用常驻守护服务 `football-insight-auto-sync.service`，自动完成拉取、`18092` 后端重启与微信小程序体验版（`robot 2`）上传，并通过 `gh api` 将执行状态回传至 GitHub Commit Status：

### 1. 在 local233 首次启用（只需执行一次）

```bash
cd /home/betalpha/projects/football_insight
git pull --ff-only

# 确保已安装并登录 GitHub CLI（用于向云端回传构建/上传状态）
gh auth status || gh auth login

# 安装并启动用户级 systemd 自动同步服务
./deploy/local-test/install-auto-sync-service.sh
```

常用运维命令（在 `local233` 执行）：

```bash
# 查看实时同步、后端重启与小程序上传日志
journalctl --user -u football-insight-auto-sync.service -f

# 单次手动触发检查
./deploy/local-test/auto-sync-local233.sh --once
```

### 2. 自动触发与状态回传规则

- **后端变更**（`football_insight_service_backend_rs/**`）：
  - 若包含 `migrations/*.sql` 变更，先自动执行 `cargo run --bin run_migrations`
  - 自动调用 `./deploy/local-test/up.sh` 编译并重启 `18092` 后端，校验 `/api/health` 等接口
  - 通过 `gh api` 回传状态上下文 `local233/backend-18092`（`pending` → `success` / `failure`）
- **小程序变更**（`football_insight_mini/**`）：
  - 若依赖文件变化，先执行 `bun install --frozen-lockfile`
  - 先同步审核版本号（若分配了新版本号，自动提交推送 `chore(mini): sync allocated review version [local233]`），再执行 `bun run mp:release -- --robot 2 --desc "<commit 标题>"`
  - 通过 `gh api` 回传状态上下文 `local233/mp-release-robot2`（`pending` → `success` / `failure`）

### 3. 云端 Agent 监控与 GitHub Actions 备用上传

- 云端 Agent 在 `git push` 后可执行：
  ```bash
  ./deploy/local-test/wait-local233-sync.sh
  ```
  实时轮询 `local233` 回传的 Commit Status，确认 `18092` 后端重启和体验版上传完成后再通知用户。
- 若需要将微信上传密钥托管到 GitHub Actions 云端作为备用通道（在 `local233` 执行一次即可注入仓库 Secrets）：
  ```bash
  cd /home/betalpha/projects/football_insight/football_insight_mini
  gh secret set MINI_CI_PRIVATE_KEY < private.wxc61da17a97f6eb1b.key
  grep '^MINI_REVIEW_API_KEY=' .env.ci.local | cut -d= -f2- | gh secret set MINI_REVIEW_API_KEY
  ```
  配置后可通过 `gh workflow run mp-release.yml -f desc="变更说明"` 直接在 GitHub 云端触发体验版上传。

