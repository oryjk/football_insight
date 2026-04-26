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

- 生产服务器：`jd`，可以 ssh jd 上去
- 项目目录：`/root/projects/football_insight_service_backend_rs`
- 不要随意改 Nginx 非 football 路由
- 生产后端使用 systemd 管理：`football-insight.service`
- 改完后端通常只需要在生产目录执行 `cargo build --release`，再 `systemctl restart football-insight.service`
- 不要再用裸 `cargo run`、前台进程或只依赖 SSH 会话的后台进程托管生产后端
- 只有修改 systemd unit 文件后才需要 `systemctl daemon-reload`
- 只有修改 Nginx 配置后才需要 reload/restart Nginx；普通后端发布不需要重启 Nginx
- 发布后至少验证：
  - `systemctl status football-insight.service --no-pager`
  - `curl -i http://127.0.0.1:8092/`
  - 一个经 Nginx 转发的线上 API，例如 `curl -k -i https://match.oryjk.cn/api/v1/ticket-watch/current-board`
