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

## 当前注意点

- 榜单数据本质上是“抓取时点的赛季累计值”，不是单轮统计
- 当前球员完整主档还需要逐步从球队球员接口补齐
- 后续洞察会依赖“当前快照 vs 上一轮最终快照”的比较

## 本地开发

```bash
cd /Users/carlwang/football_insight/sina_csl_scraper
uv sync
uv run pytest
uv run sina-csl-scraper scrape --season 2026 --write-db
```
