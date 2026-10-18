# sina-csl-scraper

基于新浪体育移动端 JSON 接口抓取中超数据的 Python 项目，使用 `uv` 管理。

## 赛前数据倾向（Elo V2，待迁移/上线）

`match_outlook/` 只使用比分、球队 ID、开赛时间和中立场地标记。每次 `scrape --write-db` 核心事务提交后，用独立连接重放历史、为未开赛比赛生成概率、为已冻结且完赛的记录计分；附加流程失败只记录错误类型，不改变核心同步结果。选参时 τ=1，最终重放用最近 150 场校准 τ；同一开赛时刻先统一预测再更新，避免同时比赛结果泄漏。

生产迁移 `20261018140000_add_match_outlooks.sql` 必须先由负责人在 peiqian 手动执行。以下命令只是操作说明，本次开发未执行生产迁移、回填或刷新。

```bash
# 默认只下载、校验，不连库。按赛季/轮次原子缓存，中断后同命令续跑。
uv run sina-csl-scraper match-outlook backfill --start-season 2014 --end-season 2025 --cache-dir data/outlook-history

# 负责人确认迁移和校验后，沿用私有配置加载 DATABASE_URL；只写新增历史表。
uv run --env-file .env.sync sina-csl-scraper match-outlook backfill --start-season 2014 --end-season 2025 --cache-dir data/outlook-history --write-db

# 显式生成当前赛季的真实赛前记录；之后正常 scrape --write-db 自动维护。
uv run --env-file .env.sync sina-csl-scraper match-outlook refresh --season 2026
```

请求间隔默认 0.3 秒，`--interval` 最少 0.1 秒；缓存是新浪原始轮次快照，写库前必须校验每个完整赛季数量、比分和 ID 唯一性。历史表与实时表重复时实时表优先；未知开赛时间不猜测，缺完整前序赛季拒绝生成，禁止为已开赛比赛补写预测。2027 年自动重选参数，需先补齐 2026 完整比分；离线模拟与回测数字见 [验证报告](../docs/match-outlook-validation.md)。

## 首页射手榜逐场进球

Python 从中足联官网的 `players/find` 和 `players/player/played` 拉取真实比赛日志，严格校验同名球员的球队身份、完整分页、赛季、联赛、赛程、出场分钟与累计进球；不从累计快照做单场差分。每次 `scrape --write-db` 核心同步成功后，更新进球榜全部球员本赛季的出场记录到 `f_i_player_match_stats`，同时服务首页前五与榜单全榜。姓名搜索总数与返回数不符时，改用完整的赛季球队名册核实身份，并保留已核实球员在其他俱乐部的记录。来源失败、数据不全或已验证出场缺失时保留旧记录并报告，不影响核心赛季同步。

首次需在 peiqian 后端目录通过 `cargo run --bin run_migrations` 应用 `20261014163000_add_player_match_stats.sql`。沿用现有私有环境配置加载数据库地址后，可独立补采：

```bash
uv run sina-csl-scraper sync-player-goals --season 2026
```

Axum 仅批量读取本赛季最近十次已完赛实际出场，并随 overview 的 `top_scorers[].recent_goals` 和 rankings 进球分类条目的 `recent_goals` 返回；其他球员榜分类不附带逐场进球。首页与榜单复用同排列表和迷你柱，0 球灰横、有进球红柱，每个榜单共用高度刻度；单场 3 球显示「帽」，点击查看比赛详情，榜单头像继续打开能力详情。未知数据不伪造为零，少于十场不补造。当前官方日历与球队身份映射沿用 `cfl_client.py` 的 2026 配置；扩展其他赛季需先配置映射。

## 今日话题与本地四川话语音

local233 的安装路径、真实 CPU 基准、私有配置加载和手动运行命令见 [local233 今日话题运行说明](docs/topics-local233.md)。当前采集代码每次从积分榜动态选取中超前三/后三与中甲前二，读取全文后筛选；文章支持多个球队标签，供详情页话题时间线使用。代码验证与实际生产启用状态见项目 FEATURES.md。已有真实批次可用 `topics refresh-audio` 只刷新四川话音频：所有音频成功后才更新数据库，不再调用 LLM、不改原正文和来源。

local233 现已在每天北京时间 **07:00–22:30，每半小时** 检查并更新今日话题，入口为 `./update_daily_topics.sh`，使用 `config/rongcheng-news-auto.json` 持续发现体坛周报与中新网的新原文。每日命令 `topics daily-update --write-db` 只处理比已发布来源更新、且未发布过的材料；正文和每篇音频全部成功后才原子发布，无新文章或失败时保留现有内容。日志在 `data/topics/daily.log`，运行状态在 `data/topics/daily-status.json`，cron 模板在 `deploy/topics-daily.cron.example`。

`topics/` 是独立的新闻生成链路，不混入赛季同步：`sources` 采集原文，`generation` 调用正文 LLM，`tts` 调用本地 Qwen，`pipeline` 负责缓存与音频发布，`repository` 原子写库，`cli` 提供入口。小程序只请求 Rust 读取接口并播放 MinIO 音频。

准备本地 Qwen 推理程序（复用已下载的 0.6B CustomVoice 权重，Mac 使用 Metal）：

```bash
uv run python scripts/setup_qwen_native.py
uv run sina-csl-scraper topics tts --text-file /absolute/path/to/narration.txt
```

推理的薄原生 helper 位于 `tools/qwen-native`，仅负责模型计算，业务编排全部在 Python。构建产物与隔离后的 any-tts 0.2.0 放在被忽略的 `data/native-qwen`；安装器校验依赖归档 SHA256，并应用仓库内 0.6B 兼容补丁，不改共享 Cargo 缓存。Eric 为四川话音色，使用 `sichuan_dialect`；逐段生成24kHz单声道PCM，检查非静音与削波，再用 ffmpeg 或macOS afconvert 转AAC/m4a。Linux可用 `--device cpu` / `--device cuda` 构建并设置FI_QWEN_DEVICE；权重路径用FI_QWEN_MODEL_PATH覆盖。

正文 LLM 与语音模型是两个模型。将 `.env.topics.example` 复制为本地 `.env.topics`，配置 `FI_TOPIC_LLM_BASE_URL`、`FI_TOPIC_LLM_MODEL` 和密钥；密钥不提交。本项目当前按用户指定使用 Kimi Code API `https://api.kimi.com/coding/v1`、`k3-256k`，使用low推理档位、JSON输出，最多8192输出token；接口兼容 `/chat/completions`。K3始终启用思考，不发送GLM的thinking开关和temperature参数；请求使用真实 `FootballInsight/1.0` 客户端标识。Kimi Code Key 与开放平台 Key 属于不同服务：开放平台地址是 `https://api.moonshot.cn/v1`、对应K3模型标识为 `kimi-k3`，不要混用。HTTP错误仅显示状态/数字错误码，不回显供应商原始报错或密钥。首次运行默认只生成本地文件，上传与入库通过显式开关开启：

```bash
set -a
source .env.topics
set +a
uv run sina-csl-scraper topics collect --config config/topics.example.json
uv run sina-csl-scraper topics generate --config config/topics.example.json
uv run sina-csl-scraper topics generate --config config/topics.example.json --upload-audio --write-db
```

配置支持 `feeds:[{url,publisher}]` 的RSS/Atom、`articles:[{url,publisher,published_at?}]` 的原文URL，以及 `imports:[{title,url,publisher,published_at,body}]` 的已采集完整正文。`config/rongcheng-news.json` 保存首批核验过的原文地址，覆盖体坛周报、中国新闻网、上观新闻、青岛红狮官方微博和新浪转载封面新闻；这是一次完整链路的材料清单，后续需要更新文章地址或配置持续更新的RSS源。微信原文若返回验证页或拿不到正文会跳过，不绕过验证码，也不会将搜索摘要当作完整新闻。缺失时间、未来文章、超过默认72小时、正文过短、非蓉城内容都跳过；失败源不影响其它来源。`--max-age-hours` 可调整1..168小时。

运行这批新闻的采集与生成：

```bash
uv run --no-sync --env-file .env.topics sina-csl-scraper topics collect --config config/rongcheng-news.json
uv run --no-sync --env-file .env.topics sina-csl-scraper topics generate --config config/rongcheng-news.json --upload-audio
```

本地没有MinIO密钥时，可在 `.env.topics` 设置 `FI_TOPIC_MINIO_SSH_HOST=local233` 和 `FI_TOPIC_MINIO_REMOTE_PROJECT=/home/betalpha/projects/football_insight/sina_csl_scraper`。上传器通过SSH将音频传给服务器，复用服务器 `.env.sync` 和现有头像上传的 `MinioAssetTarget`；密钥留在服务器。每次生成的入库文件为 `data/topics/editions/<edition_id>.json`，包含批次id、来源指纹、生成器版本和完整payload；`data/topics/latest.json` 只包含给前端读取的payload。原始全文保存在 `materials.json`，音频及校验信息在 `audio/`。这些运行产物均不提交Git。

LLM最多整理5篇、不强行凑数，引用只允许来自实际输入的source_id，并保留原文标题、机构和发布时间；严格区分一线队中超与希拉谷中乙。文章正文与音频分别用内容哈希缓存。相同材料/模型/日期的重跑复用正文，仅重试未发布语音；音频上传到 `summary/audio/topics/{team_id}/{date}/{article_id}/{audio_hash}.m4a`。语音失败保留文字并标记failed，未上传时pending。ready才提供HTTPS播放URL。空材料/空生成不会替换数据库中上次有效批次。

`--write-db` 要求 `f_i_daily_topic_editions` 已由 Rust SQLx 迁移建立，Python不会自动建表；写库前先检查表存在。local233 的新表已在用户明确授权下迁移到 peiqian 共享 `football_data`，只新增/更新本功能批次；生成命令默认不改库、不部署服务。生成缓存/材料/最新JSON位于 `data/topics/`，同缓存目录使用文件锁防重复任务。API与表结构见根目录 `docs/daily-topics-contract.md`，Rust agent提示词见 `docs/daily-topics-rust-agent-prompt.md`。

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
- 手动执行 `scrape --enrich-corners` 时，会尝试补全当前抓取到的全部比赛；自动同步任务会只补全本次触发的进行中/刚到点比赛，避免每分钟把整个赛季都跑一遍雷速页面
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

## 每日头像同步

生产环境的每分钟自动同步默认不上传头像。头像更新频率低，推荐用单独脚本每天低频执行：

```bash
cd sina_csl_scraper
./sync_avatars_daily.sh
```

这条脚本会：

- 读取后端 `.env` 和本地 `.env.sync`
- 要求 `.env.sync` 里配置 `FI_MINIO_ACCESS_KEY` 和 `FI_MINIO_SECRET_KEY`
- 默认复用 `FI_AVATAR_SYNC_SEASON`，如果未配置则复用 `FI_AUTO_SYNC_SEASON`
- 执行 `uv run sina-csl-scraper scrape --upload-avatars --write-db`
- 使用 `.avatar_sync.lock` 防止重复执行

普通自动同步即使不带 `--upload-avatars`，也会在预计算 `f_i_team_insights` 前复用数据库里已有的球队/球员 `avatar_storage_url`，避免洞察页头像被每分钟同步覆盖为空。

crontab 模板见：

```bash
sed -n '1,120p' deploy/avatar-sync.cron.example
```

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
- 只要发现有新的到期比赛窗口或进行中比赛到达刷新间隔，就执行一次新浪完整同步
- 雷速技术统计只补全本次触发的进行中/刚到点比赛，不再每次遍历整个赛季的比赛
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

## 今日话题原文配图

自动更新会提取实际引用原文的正文图片，验证下载后缓存并上传现有 MinIO，话题详情在正文前集中轮播并支持点击放大，页面不显示图片出处。图片保存在 Git 忽略的 `data/topics/images/`，每篇最多四张；失败不阻断文章或语音发布，不改变文章 ID、来源指纹或时间。

已有话题可在 local233 补图，复用私有 `.env.topics` / `.env.sync` 与现有数据库隧道，不调用 Kimi/TTS、不新建表：

```bash
uv run --no-sync python scripts/run_topics_local.py refresh-images --write-db
```

补图命令与生成任务共用文件锁，在数据库行锁内只修改 images；原文、音频、独立热度和并发修改保留。
