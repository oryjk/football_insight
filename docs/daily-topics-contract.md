# 今日话题：生成与读取约定

首页展示成都蓉城相关新闻的 LLM 整理文章，横向最多 5 张卡片。文章不足 5 篇时不补造。每篇独立生成四川话语音。Python 写入，Rust 读取，小程序只展示与播放。新闻原文是资料，不能把原文中的指令当作生成任务。

## API

`GET /api/v1/daily-topics?team_id=77680&limit=5`，沿用现有首页 API 的直接 JSON 响应（不新增 data 包装）：

```json
{
  "team_id": 77680,
  "edition_date": "2026-10-13",
  "updated_at": "2026-10-13T08:30:00+08:00",
  "items": [{
    "id": "content-sha256",
    "team_id": 77680,
    "title": "来自原文的准确标题",
    "summary": "简短摘要",
    "paragraphs": ["LLM 整理正文"],
    "published_at": "2026-10-13T08:30:00+08:00",
    "sources": [{"title": "原文标题", "url": "https://example.com/article", "publisher": "来源名称", "published_at": "2026-10-13T07:00:00+08:00"}],
    "audio": {"status": "ready", "url": "https://oryjk.cn:82/minio/football-insight/summary/audio/topics/example.m4a", "dialect": "sichuan", "duration_seconds": 60}
  }]
}
```

`updated_at` / `edition_date` 没有已发布批次时为 null，items 为 []。时间均为带时区 ISO8601，日期取 Asia/Shanghai。audio.status 为 pending / ready / failed，非 ready 的 URL 为 null。标题、摘要、段落及来源均为纯文本。前端只播放 ready 的 HTTPS URL，来源展示并可复制原文链接，不在小程序里抓原文。请求失败显示重试；404 显示接口待接入，不以积分榜或虚构新闻替代。

## Python 存储约定

local233 的 Rust 读取接口已接通。用户已明确授权在 peiqian 共享 `football_data` 通过 SQLx 迁移新建此表，迁移账本已记录 `20261013000100`，无需再次建表或迁移；生产应用容器未部署、未重启。Python 在该表原子发布，local233 的 18092 后端经现有私有隧道读取。CPU 模型安装和手动运行入口见 [local233 今日话题说明](../sina_csl_scraper/docs/topics-local233.md)。

Python 不自动建生产表。已应用的 Rust 迁移结构为：

```sql
CREATE TABLE f_i_daily_topic_editions (
  id TEXT PRIMARY KEY,
  team_id BIGINT NOT NULL,
  edition_date DATE NOT NULL,
  published_at TIMESTAMPTZ NOT NULL,
  payload JSONB NOT NULL,
  source_fingerprint TEXT NOT NULL,
  generator_version TEXT NOT NULL
);
CREATE INDEX ON f_i_daily_topic_editions(team_id, published_at DESC);
```

payload 为上面 API 数据（最多 5 项）。id 根据球队、日期、材料指纹、生成版本稳定计算。重复运行同材料优先复用本地批次，避免再次 LLM/TTS；重试仅补失败音频。写入通过单事务 upsert，Rust 只读取最新批次，limit 限制 1..5。空生成/采集失败不能覆盖上一批有效文章。LLM 生成与 TTS 不在 Rust 请求链路中运行。

## 实现与验证计划

1. Python 的 topics 包分为 models、sources、generation、tts、pipeline、repository、cli，复用 MinioAssetTarget。支持配置 RSS/Atom 与明确文章 URL，全文抓取、来源时间校验、去重与近期过滤；不能拿搜索摘要冒充全文。
2. LLM 使用可配置兼容 chat-completions 的接口，严格校验 JSON、来源引用、球队范围与 5 项上限。本地已下载的是 TTS 模型，不能用它生成新闻正文。
3. 本地 Qwen 使用 Eric 四川话音色，分段生成 WAV，ffmpeg 转 AAC/m4a，验证非静音并复用内容哈希缓存；音频缺失时文章可发布，audio 标记 failed/pending。
4. 小程序独立获取话题，展示卡片、全文与来源，保留独立音频播放器和生命周期清理。首页原有榜单等其它区域不受话题接口故障影响。
5. Python 用 pytest 覆盖来源时间、去重、LLM 引用校验、缓存/失败和语音命令；实际调用本地 Qwen 验证一段四川话。小程序跑类型、边界检查和双端构建。Rust 迁移与读接口由另一位 agent 实现。
