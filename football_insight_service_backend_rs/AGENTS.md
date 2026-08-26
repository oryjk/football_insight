# football_insight_service_backend_rs Guide

这是 Football Insight 的 Rust 后端。

## 技术栈

- Rust
- Axum
- SQLx
- PostgreSQL
- 六边形架构

## 目录规则

- `domain`
- `ports`
- `application`
- `adapters`

不要把业务逻辑直接塞进 web handler 或 SQL 层。

## 当前核心职责

- 从 PostgreSQL 读取足球数据
- 提供 H5 所需 API
- 处理账号系统
- 处理公众号 webhook
- 提供小程序审核版本登记库（`mini_review` 模块）

## 小程序审核版本登记（mini_review）

- 表：`f_i_mini_review_statuses`（迁移 `20260826120000_add_mini_review_statuses.sql`），登记库是版本号唯一权威
- `GET /api/v1/mini-review/review-status?project_code=&version=`：小程序运行时公开查询，未登记版本视为不在审核
- `POST /api/v1/mini-review/allocate`：构建脚本申请版本号（最新版本审核中→复用；已出审核→+0.0.1；空库→以 manifest 为起点递增）
- `PUT /api/v1/mini-review/review-status`：微信审核结束后标记 `is_reviewing=false`
- allocate 与 PUT 用请求头 `X-Api-Key` 静态鉴权，密钥来自后端 env `MINI_REVIEW_API_KEY`（为空时接口不开放）；与 `football_insight_mini/.env.ci.local` 同名键保持一致


## 当前账号体系

- 邀请码注册
- 用户名或手机号密码登录
- JWT 鉴权

用户相关接口才需要 JWT。
公开数据接口默认不鉴权。

## 微信相关现状

- `GET/POST /football/wechat/webhook` 已接通
- 关注公众号会生成邀请码
- 个人主体公众号不适合作为网页 OAuth 登录主入口
- 即使代码支持 `snsapi_base`，如果公众号类型没有权限，微信仍会直接报 scope 错误

## 本地开发

```bash
cd /home/wangrui/projects/football_insight/football_insight_service_backend_rs
cargo test
cargo check
cargo run --bin football_insight_service_backend_rs
```

重要约定：

- 只要修改了后端代码，本地接口验证前必须重启本地后端进程
- 不要在旧的 `cargo run --bin football_insight_service_backend_rs` / 已存在的 8092 进程上验证新代码
- 遇到“代码已经改了但接口仍是旧行为”，先检查并重启本地后端，再继续排查

## 部署注意

- 默认生产服务器：`jd`，可以 ssh jd 上去
- 备用生产服务器：`peiqian`，可以 `ssh peiqian` 上去
- 生产 monorepo 目录：`/root/projects/football_insight`
- 项目目录：`/root/projects/football_insight/football_insight_service_backend_rs`
- 不要随意改 Nginx 非 football 路由
- 生产后端优先使用 Docker 管理：容器名 `football-insight-service-backend-rs`
- 部署到 `jd` 时运行 `./deploy_jd_docker.sh`
- 部署到 `peiqian` 时运行 `./deploy_peiqian_docker.sh`
- systemd `football-insight.service` 保留为备用部署方式
- 不要再用裸 `cargo run`、前台进程或只依赖 SSH 会话的后台进程托管生产后端
- 只有修改 systemd unit 文件后才需要 `systemctl daemon-reload`
- 只有修改 Nginx 配置后才需要 reload/restart Nginx；普通后端发布不需要重启 Nginx
- 当前已确认 `peiqian` 本机 8092 在监听且容器健康；`match.oryjk.cn` 仍对应 `jd`，两台机器通过 WireGuard 联通
- 发布后至少验证：
  - `docker ps --filter name=football-insight-service-backend-rs`
  - `curl -i http://127.0.0.1:8092/`
  - 一个经 Nginx 转发的线上 API，例如 `curl -k -i https://match.oryjk.cn/api/v1/ticket-watch/current-board`
