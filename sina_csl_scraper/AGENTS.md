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

- local233 用户 crontab 每天北京时间 07:00–22:30 每半小时运行 `./update_daily_topics.sh --scheduled`，23:00 截止自动执行。
- 来源为 `config/rongcheng-news-auto.json` 的体坛周报滚动列表与中新网体育 RSS，必须获取完整原文与真实发布时间；每次按当前积分榜动态选中超前三/后三和中甲前二，不按列表标题关键词拦截。
- 最近 72 小时与已发布来源链接去重，查询快照时间不参与新闻新旧判断；文章支持多球队标签，旧文补标签仅修改标签。新材料的正文与全部音频 ready 后才原子发布；无新文章或生成/上传/写库失败保留原内容，待处理批次可跨天续跑。
- 日志/状态/断点在 Git 忽略的 `data/topics/daily.log`、`daily-status.json`、`daily-state.json`；凭据继续使用私有 `.env.topics` / `.env.sync`，不自动迁移。

## 当前注意点

- 榜单数据本质上是“抓取时点的赛季累计值”，不是单轮统计
- 当前球员完整主档还需要逐步从球队球员接口补齐
- 球队洞察已实现，不是靠“当前快照 vs 上一轮最终快照”差分，而是按积分榜 + 球队榜/球员榜 + 已完场比赛直接累计
- 硬约束：每轮必须恰好 8 场比赛（`expected_matches_per_round`，默认 8，由 `sync.py` 的 `validate_complete_schedule` 校验），不满足时 sync 事务回滚

## 本地开发

```bash
cd sina_csl_scraper
uv sync
uv run pytest
uv run sina-csl-scraper scrape --season 2026 --write-db
```
