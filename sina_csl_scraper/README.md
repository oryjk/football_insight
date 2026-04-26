# sina-csl-scraper

基于新浪体育移动端 JSON 接口抓取中超数据的 Python 项目，使用 `uv` 管理。

当前会导出 7 类核心数据：

- 每场比赛结果
- 积分榜
- 球队数据统计榜
- 球员数据排行榜
- 球队主档
- 球员主档
- 球队洞察预计算结果

当前还支持可选补充：

- 比赛维度角球数据（基于雷速详情页的 best-effort 补全）

## 数据来源

项目直接调用新浪体育移动端接口，而不是解析 PC 页面 HTML。当前使用的接口包括：

- `https://goal.sports.sina.cn/op/api/league`
- `https://goal.sports.sina.cn/op/api/standings`
- `https://goal.sports.sina.cn/li/api/schedule/round`
- `https://goal.sports.sina.cn/op/api/ranking/team`
- `https://goal.sports.sina.cn/op/api/ranking/player`

角球补全当前依赖的雷速页面规则：

- 实时页：`https://live.leisu.com/`
- 完场页：`https://live.leisu.com/wanchang-YYYYMMDD`
- 详情页：`https://live.leisu.com/detail-{detail_id}`

例如 `2026-04-17` 的完场页就是：

- `https://live.leisu.com/wanchang-20260417`

## 安装

```bash
cd sina_csl_scraper
uv sync
uv run playwright install chromium
```

## 运行

抓取当前赛季：

```bash
uv run sina-csl-scraper scrape
```

当前默认赛季会从新浪接口读取，目前返回的是 `2026`。

抓取指定赛季并输出到自定义目录：

```bash
uv run sina-csl-scraper scrape --season 2026 --output-dir ./exports
```

如需同时写入 PostgreSQL：

```bash
FI_DATABASE_URL='postgresql://football_app:password@127.0.0.1:55432/football_data?sslmode=disable' \
uv run sina-csl-scraper scrape --season 2026 --write-db
```

如需同时尝试补全角球数据，可以直接开启自动补全：

```bash
uv run sina-csl-scraper scrape \
  --season 2026 \
  --enrich-corners
```

当前自动补全会：

- 完场比赛优先从 `wanchang-YYYYMMDD` 页面按日期发现雷速 `detail_id`
- 实时比赛优先从雷速首页 `https://live.leisu.com/` 发现 `detail_id`
- 通过 Playwright 无头浏览器加载页面，尽量绕过雷速的前端 WAF challenge
- 拿到 `detail_id` 后，再进入详情页按角球事件文本累计主客角球

如果你已经有更稳定的雷速映射，也可以额外提供一个 JSON 文件，把新浪 `match_id` 映射到雷速 `detail_id`，它会优先覆盖自动发现结果：

```json
{
  "288579": 4422785
}
```

示例：

```bash
uv run sina-csl-scraper scrape \
  --season 2026 \
  --enrich-corners \
  --leisu-match-map ./leisu_match_map.json
```

当前角球补全规则：

- scraper 仍然以新浪赛程为主数据源
- 角球补全是 best-effort，不会阻塞原有抓取
- 当前通过雷速完场页 / 实时页发现详情页，再用详情页里的角球事件文本累计主客队角球数
- 如果自动发现失败、没有映射、详情页抓取失败、或页面里没有可解析角球事件，则 `home_corners` / `away_corners` 保持为空
- `matches.json` 会额外写出 `leisu_match_id`、`home_corners`、`away_corners`、`corner_source`
- PostgreSQL 只有在 `f_i_matches` 已存在 `leisu_match_id`、`home_corners`、`away_corners`、`corner_source` 四列时才会写入这些字段；否则自动兼容旧表结构

也可以把头像上传和数据库写入一起执行：

```bash
FI_DATABASE_URL='postgresql://football_app:password@127.0.0.1:55432/football_data?sslmode=disable' \
FI_MINIO_ENDPOINT=https://example.com/minio \
FI_MINIO_ACCESS_KEY=change_me \
FI_MINIO_SECRET_KEY=change_me \
FI_MINIO_BUCKET=football-insight \
FI_MINIO_REGION=us-east-1 \
FI_MINIO_PREFIX=summary \
uv run sina-csl-scraper scrape --season 2026 --upload-avatars --write-db
```

为了避免每次手动敲一长串环境变量，项目根目录提供了一个可执行脚本：

```bash
cd sina_csl_scraper
cp .env.sync.example .env.sync
# 按需修改 .env.sync 中的 MinIO 配置
./sync_latest.sh 2026
```

如果不传赛季参数，脚本会直接使用 scraper 默认逻辑读取当前赛季：

```bash
./sync_latest.sh
```

脚本会自动：

- 读取后端 `.env` 里的 `DATABASE_URL`
- 读取本地 `.env.sync` 里的 MinIO 配置
- 执行 `uv run sina-csl-scraper scrape --upload-avatars --write-db`

## 自动同步

如果需要“每场比赛按开球时间推导结束时间，再延后 10 分钟自动同步一次”，项目根目录还提供了：

```bash
cd sina_csl_scraper
./auto_sync_latest.sh
```

这条脚本会：

- 读取后端 `.env` 里的 `DATABASE_URL`
- 读取本地 `.env.sync` 里的 MinIO 配置
- 默认开启雷速技术统计补全，包括角球
- 默认按“比赛开始 + 120 分钟 + 10 分钟”判断是否到达同步窗口
- 只要发现有新的到期比赛窗口，就执行一次完整同步
- 把上次已处理窗口记录到 `.auto_sync_state.json`，避免重复拉取同一场
- 如果系统安装了 `flock`，脚本会自动加锁，避免每分钟重复触发时并发跑多份进程

可先用 dry run 看当前是否会触发：

```bash
./auto_sync_latest.sh --dry-run
```

如果你要部署到 Linux 服务器上每分钟执行一次，推荐用仓库里的模板：

```bash
sed -n '1,120p' deploy/auto-sync.cron.example
```

当前生产定时任务跑在 `local233`，路径约定是：

```text
/home/betalpha/projects/football_insight/sina_csl_scraper
```

部署前先准备运行时环境变量：

```bash
cd sina_csl_scraper
cp .env.sync.example .env.sync
```

可以按需调整这些变量：

- `FI_AUTO_SYNC_SEASON=2026`：固定抓 `2026` 赛季
- `FI_AUTO_SYNC_WRITE_DB=1`：自动写 PostgreSQL
- `FI_AUTO_SYNC_UPLOAD_AVATARS=0`：默认关闭头像上传，避免每分钟任务依赖 MinIO
- `FI_AUTO_SYNC_ENRICH_CORNERS=1`：默认开启雷速技术统计补全

服务器上先安装浏览器依赖：

```bash
uv sync
uv run playwright install chromium
```

用 cron 每分钟执行一次：

注意：

- `pyproject.toml` 当前要求 `Python >=3.12`
- local233 当前使用用户 crontab 调度，不使用 systemd timer

示例：

```cron
* * * * * /bin/zsh -lc 'cd /home/betalpha/projects/football_insight/sina_csl_scraper && ./auto_sync_latest.sh >> /home/betalpha/projects/football_insight/sina_csl_scraper/auto_sync.log 2>&1'
```

当前默认规则：

- 假设比赛结束时间 = 开球时间 + 120 分钟
- 自动同步触发时间 = 结束时间 + 10 分钟
- cron 本身每 1 分钟检查一次，所以实际触发时点会更接近真实的赛后窗口

## 输出文件

默认会写入 `data/<season>/`：

- `league_info.json`
- `standings.json`
- `matches.json`
- `team_rankings.json`
- `player_rankings.json`
- `teams.json`
- `players.json`

其中 `matches.json` 现在可能包含以下附加字段：

- `leisu_match_id`
- `home_corners`
- `away_corners`
- `corner_source`

## 球队洞察预计算

当前“洞察页”里的这些内容，计算职责放在 scraper：

- 进球贡献
  - 对手维度：这支球队的进球主要打在谁身上
  - 球员维度：本队球员对总进球的贡献占比
- 助攻贡献
  - 球员维度：本队球员对总助攻的贡献占比
- 失球贡献
  - 对手维度：哪些对手对这支球队造成了更多失球

实现入口在：

- [team_insights.py](src/sina_csl_scraper/team_insights.py)

写库入口在：

- [postgres_repository.py](src/sina_csl_scraper/postgres_repository.py)

当前规则：

- scraper 每次同步后，按“球队维度”预计算这些贡献结果
- 结果写入 PostgreSQL 的 `f_i_team_insights`
- 如果球员维度加总不足总进球/总助攻，会补一条 `其他/未归因`
- 前端默认只显示前 3 条；是否展示“更多”属于 H5 展示逻辑，不属于 scraper 计算逻辑

也就是说：

- scraper 负责算并落库
- Rust 后端只读 `f_i_team_insights`
- H5 请求的是已经算好的结果，不会临时计算

如需同时下载新浪头像并上传到 MinIO：

```bash
FI_MINIO_ENDPOINT=https://example.com/minio \
FI_MINIO_ACCESS_KEY=change_me \
FI_MINIO_SECRET_KEY=change_me \
FI_MINIO_BUCKET=football-insight \
FI_MINIO_REGION=us-east-1 \
FI_MINIO_PREFIX=summary \
uv run sina-csl-scraper scrape --season 2026 --upload-avatars
```

默认情况下，程序会根据 `FI_MINIO_ENDPOINT` 和 `FI_MINIO_BUCKET` 自动生成公开地址前缀：

- `https://example.com/minio` + `football-insight`
- 推导为 `https://example.com/minio/football-insight`

如果你的 MinIO 反代或 CDN 地址不同，再显式传 `FI_MINIO_PUBLIC_BASE_URL`。

注意：当前 bucket 需要对 `summary/*` 开启匿名读，前端直接访问 `avatar_storage_url` 才不会返回 `403`。如果暂时不开放匿名读，仍然可以使用 `avatar_object_name`，由后端代理读取 MinIO 后再返回给前端。

## 开发

运行测试：

```bash
uv run pytest
```
