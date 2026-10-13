# 今日话题后端验证记录（2026-10-13）

## 实现与范围

- 新增 `src/daily_topics` 的 domain/ports/application/persistence/web/bootstrap 分层；在 `src/lib.rs` 导出、`src/app.rs` 注册公共 GET 路由。
- 新增 SQLx 迁移 `20261013000100_add_daily_topic_editions.sql`，仅创建独立表和索引，不修改既有数据或表。
- `src/http_cache.rs` 仅排除新路由，避免批次或音频状态更新被原有 600 秒公共缓存遮挡。
- 文档新增 `docs/daily-topics.md`；README 追加新接口入口。首次 Rust 实现未修改 Python、小程序及既有认证/支付/ticket-watch 业务逻辑；后续 Python 手动语音链路见文末。

## 测试与审查

- 新 HTTP 测试首先因模块缺失失败；实现后通过。缓存回归测试先证明发布后仍返回缓存的 0 篇文章，再通过排除新路由修复。
- 最终 `cargo test`：356 passed，5 ignored。5 个明确要求临时 PostgreSQL 的集成测试另行运行：5 passed，0 failed。总计 361 项通过。
- PostgreSQL 测试覆盖空库、球队隔离、按发布时间选择最新批次、原文/音频字段与稳定 id、同批次 JSONB 原子 upsert、条数限制，以及损坏 payload/缺表失败。每个用例使用独立 schema，测试后删除；临时 PostgreSQL 容器与私有覆盖文件已清理。
- `cargo check`、本次改动 Rust 文件的 `rustfmt --check`、`git diff --check` 均通过；新模块单独执行六边形边界扫描通过。
- 全仓 `cargo fmt --check` 因已有 auth/mini_review 共 11 个文件的格式差异失败；全仓严格 `cargo clippy --all-targets -- -D warnings` 报 23 个已有诊断，包括 too_many_arguments、collapsible_if、new_without_default、unused Result 等，未命中新模块。全仓边界扫描有两类既有问题：Web-to-Web DTO 引用、按错误字符串分支。没有为通过这些检查扩大修改范围。
- 独立只读代码审查未发现 Critical/Important 问题；指出新路由缓存继承风险，已以回归测试修复。

## 数据库与部署

- 18092 原主 DATABASE_URL 经 SSH 隧道连接 peiqian 的共享生产 `football_data`。用户明确授权在原库按 SQLx 流程新增表，撤回单独运行数据库方案。
- 应用前核验原有 46 条迁移 checksum 一致、唯一待执行迁移为 `20261013000100`，目标表此前不存在。
- 在 peiqian 执行 `cargo run --bin run_migrations` 应用新增迁移，由 `_sqlx_migrations` 记账。生产应用容器的 image 与 StartedAt 前后相同，未部署、未重启。
- 本地重建最终镜像 `football-insight-local-test-backend:daily-topics-20261013`，镜像 ID 为 `sha256:f0c94c56a7469af253629443822a1c3ca4fde8b583ef20d7752d57b3af1c6080`，构建源码摘要为 `e7d62f502f45cba25e356f6035e1492ac817798ec4183bf9ab77d22bee967967`。
- 真正承接 18092 的容器 `football-insight-local-test-backend` 已重建/重启，StartedAt 为 `2026-10-13T00:55:18.162383836Z`（北京时间 08:55）。所有运行环境变量与端口映射保持原值，保留 Git 忽略的微信真实凭据覆盖文件；未打印或提交密钥。

## 实际 HTTP 验证

```text
GET /api/health                                              200
GET /api/v1/daily-topics?team_id=77680&limit=5                 200
GET /api/v1/daily-topics                                     200
GET /api/v1/daily-topics?team_id=999&limit=1                   200
GET /api/v1/daily-topics?team_id=0                            400
GET /api/v1/daily-topics?limit=6                              400
GET /api/v1/ticket-watch/matches                             200（48 场）
GET /api/v1/ticket-watch/regions                             200（71 区域）
GET /api/v1/system/public-config                             200
容器 -> host.docker.internal:4000/api/health/live             200
```

新接口未经鉴权且没有 `data` 包装，没有 X-Cache 命中。初次接通时的真实响应：

```json
{"team_id":77680,"edition_date":null,"updated_at":null,"items":[]}
```

这表示初次验证时接口与数据库已接通、等待 Python 发布真实批次。没有读取 Python 本地 latest.json 或写入测试新闻。微信登录用不可用的诊断 code 验证配置仍返回 `40029 invalid code`，不再出现 invalid appid；没有声称以真实 wx.login code 完成用户登录。

后续真实链路已接通：Python 发布的 2026-10-13 批次包含 2 篇真实 Kimi 整理文章，已在 local233 用 CPU Qwen Eric 四川话重新生成每篇独立音频；数据库与 18092 JSON、limit=1、MinIO HTTP 200 / Range 206 / SHA 均一致。完整运行与 CPU 实测见 [local233 运行说明](../../sina_csl_scraper/docs/topics-local233.md)。该阶段没有再次迁移或重启生产后端。
