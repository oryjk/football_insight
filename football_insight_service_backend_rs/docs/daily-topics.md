# 今日话题后端

Python 负责新闻采集、文章整理、四川话音频生成和 MinIO 上传。Rust 只读取 `f_i_daily_topic_editions` 已原子发布的 JSONB 批次，不在请求时采集、生成或代理音频。

公开接口：

```text
GET /api/v1/daily-topics?team_id=77680&limit=5
```

默认 `team_id=77680`、`limit=5`。球队 ID 必须为正整数，数量必须为 1–5；非法参数返回 400。按球队精确筛选，读取 `published_at` 最新的一整个批次，再截断文章数量。顶层直接返回 `team_id`、`edition_date`、`updated_at`、`items`，不增加 `data` 包装。

无已发布批次返回：

```json
{"team_id":77680,"edition_date":null,"updated_at":null,"items":[]}
```

数据库不可用或批次损坏返回 500，不伪装成空列表。接口跳过公共响应缓存，以便 Python 同批次音频从 pending 更新为 ready 后立即可读。时间使用带时区的 ISO8601，顶层发布时间输出 Asia/Shanghai 偏移；批次日期由 Python 按 Asia/Shanghai 写入 DATE。正文只以纯文本字段返回，每篇保留 id、标题、摘要、段落、发布时间和原文来源。

音频包含 `status/url/dialect/duration_seconds`。pending/failed 的 URL 一律为 null；ready 只返回无 userinfo 的 HTTPS URL。非法 ready URL 降级为 failed，不影响文章正文。小程序直接播放公共 MinIO URL，后端不读取或返回供应商密钥。

## 数据表与 Python 写入

迁移：`migrations/20261013000100_add_daily_topic_editions.sql`。

字段：`id TEXT PRIMARY KEY`、`team_id BIGINT`、`edition_date DATE`、`published_at TIMESTAMPTZ`、`payload JSONB`、`source_fingerprint TEXT`、`generator_version TEXT`，全部非空；索引为 `(team_id, published_at DESC)`。payload 保存完整接口批次 JSON。查询以数据库的球队、日期、发布时间列为权威，文章内容从 payload.items 解码。Python 以 id upsert，同批次音频更新只更新 payload；空采集或空生成不得覆盖上次有效批次。

## local233 联调与迁移

现有 `18092` 主数据库通过 SSH 隧道连接 peiqian 的生产 `football_data`，今日话题也使用该连接池。用户明确授权在此共享库新增表；先核对迁移账本，然后在 peiqian 执行 `cargo run --bin run_migrations`，由 `_sqlx_migrations` 记账。生产应用容器不部署、不重启。

临时 PostgreSQL `football_insight_daily_topics_test` 仅用于真实 SQLx 集成测试，不作为运行接口的数据源。测试使用独立 schema，结束后删除测试 schema；联调完成清理临时容器，不向共享库写测试占位文章。

本地启动时需合并两份 Compose 配置：

```text
deploy/local-test/compose.yaml
deploy/local-test/compose.wechat.local
```

`.local` 微信覆盖文件被 Git 忽略，权限 0600，包含私有运行配置；不提交、不打印其内容。重建时保留现有主 DATABASE_URL，将它在内存中传给 `LOCAL_ADMIN_DATABASE_URL`，使用 `docker compose -f ... -f ... up -d --no-deps --no-build --force-recreate backend`。不要为了部署本功能直接调用默认 `up.sh`，该脚本会执行共享生产库迁移并且不自动加载微信覆盖文件。

Python 的 TopicsRepository 写入同一个生产 `football_data` 的新表；真实批次写入后，local233 的 GET 接口即可读取。真正发布生产应用仍需独立授权并部署 peiqian 后端。

## 验证

```bash
cargo test
DAILY_TOPICS_TEST_DATABASE_URL='<isolated PostgreSQL URL>' cargo test --test daily_topics_postgres -- --ignored
```

数据库测试要求显式设置 URL，并拒绝数据库名不以 `_test` 结尾的连接。每个用例使用唯一 schema，退出时删除测试 schema；不向运行接口的 public schema 发布测试占位文章。
