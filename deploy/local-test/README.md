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
