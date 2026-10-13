# 今日话题后端实施计划

依据：2026-10-13 用户授权的接口约定。Rust 只读取 Python 原子发布的 JSONB 批次，不采集新闻，不生成文章或音频。

## 数据与接口

- 迁移建立 `f_i_daily_topic_editions`：`id TEXT PRIMARY KEY`、`team_id BIGINT`、`edition_date DATE`、`published_at TIMESTAMPTZ`、`payload JSONB`、`source_fingerprint TEXT`、`generator_version TEXT`，所有字段非空。
- 查询按 `team_id` 精确筛选，按 `published_at DESC, id DESC` 取最新批次。
- 公共 `GET /api/v1/daily-topics`：默认球队 `77680`、数量 `5`，球队必须为正整数，数量 `1..5`。
- 无批次时直接返回 `team_id/edition_date:null/updated_at:null/items:[]`；数据库或损坏批次返回 500，不能伪装空结果。
- 每篇保留稳定 id、纯文本、来源和带时区时间。pending/failed 音频不返回 URL；ready 只允许 HTTPS。

## 文件与步骤

1. 在 `tests/daily_topics_http.rs` 编写参数、空结果、条数、来源及音频、故障行为测试，记录新增模块缺失的失败。
2. 新建 `src/daily_topics/{domain,ports,application,adapters/{persistence,web}}`；bootstrap 装配并在 `app.rs` 注册。
3. 新建 `migrations/20261013000100_add_daily_topic_editions.sql`；添加仅使用显式隔离库的真实 SQLx 集成测试，验证迁移、球队隔离、最新批次与整批更新。
4. 执行 `cargo fmt --check`、`cargo test`、隔离库测试、边界检查、`cargo check` 和 clippy；已有无关失败单独记录。
5. 重建/重启真正承接 18092 的本地后端，保留微信覆盖配置与 ticket-monitor URL，验证真实 HTTP 结果。

## 数据库限制

18092 当前连接的是 peiqian 的生产共享库。用户于 2026-10-13 明确授权在现有库新增表并按 SQLx 迁移流程应用，不使用独立运行库。临时隔离 PostgreSQL 只用于集成测试，测试完成清理。生产迁移在 peiqian 执行 `cargo run --bin run_migrations`，保留 `_sqlx_migrations` 记账；先核对迁移账本，确保只应用本次新增迁移。不调用默认 `up.sh`，本地重建保留微信覆盖配置。

生产后端本次不部署、不重启。测试占位批次仅用于隔离测试 schema，测试结束删除，不发布为真实文章。
