# sina_csl_scraper Guide

这是 Football Insight 的 Python 抓取器。

## 技术栈

- Python
- uv
- httpx / requests 风格接口抓取
- PostgreSQL 写入
- MinIO 头像上传

## 当前职责

- 抓取新浪体育移动端 JSON 接口
- 生成球队、球员、比赛、积分榜、球队榜、球员榜数据
- 上传球队和球员头像到 MinIO
- 写入 PostgreSQL `football_data`

## 今日话题自动更新

- local233 用户 crontab 每天北京时间 07:00–22:30 每半小时运行 `./update_daily_topics.sh --scheduled`，23:00 截止自动执行。
- 来源为 `config/rongcheng-news-auto.json` 的体坛周报滚动列表与中新网体育 RSS，必须获取完整原文与真实发布时间。
- 新材料的正文与全部音频 ready 后才原子发布；无新文章或生成/上传/写库失败保留原内容，待处理批次可跨天续跑。
- 日志/状态/断点在 Git 忽略的 `data/topics/daily.log`、`daily-status.json`、`daily-state.json`；凭据继续使用私有 `.env.topics` / `.env.sync`，不自动迁移。

## 当前注意点

- 榜单数据本质上是“抓取时点的赛季累计值”，不是单轮统计
- 当前球员完整主档还需要逐步从球队球员接口补齐
- 后续洞察会依赖“当前快照 vs 上一轮最终快照”的比较

## 本地开发

```bash
cd sina_csl_scraper
uv sync
uv run pytest
uv run sina-csl-scraper scrape --season 2026 --write-db
```
