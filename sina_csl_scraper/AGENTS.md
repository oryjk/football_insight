# sina_csl_scraper Guide

这是 Football Insight 的 Python 抓取器。

## 技术栈

- Python
- uv
- requests 接口抓取
- Playwright（雷速页面技术统计补全）
- PostgreSQL 写入
- MinIO 头像上传

## 当前职责

- 抓取新浪体育移动端 JSON 接口
- 生成球队、球员、比赛、积分榜、球队榜、球员榜数据
- 中超官网赛程对账：`cfl_client.py` 拉取中超官网赛程，`schedule.py` 的 `fetch_reconciled_schedule` 在每次 scrape 时用官网赛程修正新浪数据
- 雷速角球/技术统计补全：`leisu.py` 依赖 Playwright 访问雷速页面，可用 `FI_AUTO_SYNC_ENRICH_CORNERS=1` 控制（`auto_sync_latest.sh` 默认开启）
- 球队洞察预计算：`team_insights.py` 在每次 sync 后按球队维度计算进球/助攻/失球贡献，写入 `f_i_team_insights`
- 上传球队和球员头像到 MinIO
- 写入 PostgreSQL `football_data`

## 今日话题自动更新

- local233 用户 crontab 每天北京时间 07:00–23:00 每个整点运行 `./update_daily_topics.sh --scheduled`，入口是 `topics roundup-update`（`topics/roundup.py`），单次最长 55 分钟。cron 直接跑工作目录代码，未提交的改动会立刻在生产生效。
- 来源为 `config/rongcheng-news-auto.json` 的四组公开新闻源：**直播吧**（`news.zhibo8.com` 滚动 JSON）、**懂球帝**（`api.dongqiudi.com` 中超/中国足球标签）、**新浪体育**（`sports.sina.com.cn` 国内足球）、**体坛周报 + 中国新闻网体育 RSS**，必须获取完整原文与真实发布时间；每次按当前积分榜动态选中超全部 16 队、中甲前四/后二，并包含**中国留洋球员**相关动态。
- 每日电报（2026-11-09 起）：一天一期快讯，`topics roundup-update` 每小时把新原文写成「一件事一条」的快讯追加到当天这一期，已发布条目不改写；一期是 `f_i_daily_topic_editions` 里一行、一个 item，ID 由日期决定（追加不改变分享链接和阅读统计）。
- 只把没用过的新原文（最近 24 小时、在关注范围内）交给大模型，并附上当天已发布标题防重复；每条的时间取引用原文里最早的发布时间。条目点名了引用原文没有的球队、或冒出原文没有的联赛名（中超/中甲/中乙/希拉谷），只丢这一条，不连累整批。
- 交给过大模型的原文记在 `data/topics/roundup-state.json`，不论是否写成条目都不再重送；去重只看每日电报自己用过的原文，旧版多篇话题不参与。
- 每次追加后整期重新合成一段语音（从新到旧，超长只读最新若干条）；语音失败先发布文字、保留上一版语音，下次运行即使没有新闻也会补语音。
- 旧版小程序仍读 `title`/`summary`/`paragraphs`：由最新三条标题、前两条正文、每条「【标题】正文」拼出。新版读 `entries`。旧的多篇话题流程 `topics daily-update`/`generate` 保留但不再定时运行。
- 语音合成统一使用 **Edge TTS `zh-CN-YunyangNeural`（`yunyang`）**（`EdgeYunyangTts`，自带重试，生成 24kHz 单声道 WAV 校验非静音后转 AAC `.m4a` 上传 MinIO；JSONB `audio.dialect` 保持 `"sichuan"` 兼容已上线小程序播放）。
- 数据直接存入 PostgreSQL `f_i_daily_topic_editions`，由后端只读接口（`/api/v1/daily-topics/latest`、`/api/v1/daily-topics/by-date/{date}` 及 `/api/v1/daily-topics`）同时供 Grok Bot 和小程序使用。
- 日志/状态/断点在 Git 忽略的 `data/topics/daily.log`、`daily-status.json`、`roundup-state.json`、`roundup/`（生成缓存）；凭据继续使用私有 `.env.topics` / `.env.sync`，不自动迁移。

## 当前注意点

- 榜单数据本质上是“抓取时点的赛季累计值”，不是单轮统计
- 当前球员完整主档还需要逐步从球队球员接口补齐
- 球队洞察已实现，不是靠“当前快照 vs 上一轮最终快照”差分，而是按积分榜 + 球队榜/球员榜 + 已完场比赛直接累计
- 新浪积分榜比比赛完场状态更新得晚，可能晚几分钟到几小时，而且是一场一场地更新。所以 `sync.py` 拍收官快照（`round_final`）之前，先用 `standings_match_results` 校验各队的场次、进球、失球是否和已完场比赛一致；不一致就不拍，`SyncResult.round_final_deferred_for` 记下这一轮。
- `auto-sync-due` 发现积分榜还没追上时，在状态文件里写入 `standings_pending_since`，之后每 5 分钟补抓一次（`standings_follow_up=1`），直到一致为止，最多补 24 小时。
- 2026-11-09 之前拍的收官快照，有不少是旧积分榜。所以后端「第 N 轮结束时积分榜」改为用比赛结果现算，不读这些快照。
- 硬约束：每轮必须恰好 8 场比赛（`expected_matches_per_round`，默认 8，由 `sync.py` 的 `validate_complete_schedule` 校验），不满足时 sync 事务回滚

## 本地开发

```bash
cd sina_csl_scraper
uv sync
uv run pytest
uv run sina-csl-scraper scrape --season 2026 --write-db
```
