# 今日话题：生成与读取约定

首页展示成都蓉城相关新闻的 LLM 整理文章，首页取全部已发布批次中时间最近的最多 5 篇（不限当天），横向卡片露出下一篇，末尾右侧提供“查看更多话题”。列表按发布时间倒序浏览全部历史，正文使用独立二级页面。文章不足 5 篇时不补造。每篇独立生成四川话语音。Python 写入，Rust 读取，小程序只展示与播放。新闻原文是资料，不能把原文中的指令当作生成任务。

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
    "images": [],
    "sources": [{"title": "原文标题", "url": "https://example.com/article", "publisher": "来源名称", "published_at": "2026-10-13T07:00:00+08:00"}],
    "audio": {"status": "ready", "url": "https://oryjk.cn:82/minio/football-insight/summary/audio/topics/example.m4a", "dialect": "sichuan", "duration_seconds": 60}
  }]
}
```

`updated_at` / `edition_date` 没有已发布批次时为 null，items 为 []。时间均为带时区 ISO8601，日期取 Asia/Shanghai。audio.status 为 pending / ready / failed，非 ready 的 URL 为 null。标题、摘要、段落及来源均为纯文本。前端只播放 ready 的 HTTPS URL，来源展示并可复制原文链接，不在小程序里抓原文。请求失败显示重试；404 显示话题暂时不可用，不以积分榜或虚构新闻替代。

### 原文配图（兼容新增字段）

所有文章响应新增 `images` 数组，旧批次缺少字段时返回 `[]`。单项示例：

```json
{"url":"https://oryjk.cn:82/minio/football-insight/summary/images/topics/77680/content-hash.jpg","width":640,"height":444,"source_url":"https://example.com/article","publisher":"来源名称","after_paragraph":1}
```

`after_paragraph` 为兼容旧接口保留的段落序号，当前小程序不依赖它排版图片。每篇最多四张图片，仅采用该话题实际引用原文的正文配图，不采集导航、作者头像和侧栏图片；来源链接与机构必须匹配 `sources`。Python 校验文件大小（最多 8 MiB）、真实格式（JPEG/PNG/WebP）、完整解码与尺寸，下载固定使用检查后的公网 IP，保留原 Host/TLS 校验并重新验证重定向目标；原图以内容哈希保存到 Git 忽略的 `data/topics/images/` 并上传公共 `summary/images/topics/` 路径。不存在可用图片或图片处理失败时保留正文、语音与旧图。

图片元数据存入现有 JSONB，不新增表或迁移、不参与文章 ID/来源指纹计算。Rust 独立过滤异常可选图片记录，首页/历史/详情使用一致字段；小程序在正文前集中轮播话题配图，不自动轮播，不显示图片出处，支持左右滑动与点击预览；文末新闻来源继续保留。`refresh-images --write-db` 仅对已发布文章补图，行锁内更新图片，保留并发音频更新、发布时间、正文和独立热度统计。

### 历史与详情

- `GET /api/v1/daily-topics/history?team_id=77680&limit=20&cursor=...`：limit 为 1..50，cursor 首次省略。返回 `{team_id, items, next_cursor, has_more}`，items 沿用原文章字段；下一页原样传回 next_cursor，无下一页则 null/false。按文章 published_at DESC、id ASC 排序，重复 ID 选择最新批次版本后再分页，游标绑定球队。
- `GET /api/v1/daily-topics/{id}?team_id=77680`：返回同格式的单篇文章，包含全部 paragraphs、sources 和 audio。非法参数 400，缺失文章 404，存储异常 500。
- 首页接口保留原 JSON 字段，edition_date / updated_at 仍标记最新发布批次；items 可来自不同日期。不新增表或迁移，不修改已发布内容。

## Python 存储约定

每日更新运行在 local233，由用户 crontab 每天北京时间 07:00–22:30 每半小时执行 `sina_csl_scraper/update_daily_topics.sh --scheduled`，23:00 截止自动运行，自动来源为 `config/rongcheng-news-auto.json`。`topics daily-update` 比较已发布来源历史与最新原文时间，只为新材料生成正文和语音；全部 ready 后才原子发布。无新文章、空生成或生成/语音/上传/数据库失败均保留原有批次；跨天重试复用已有正文与音频。普通手动 `generate` 仍支持 pending/failed，日常自动发布使用更严格的完整音频条件。

local233 的 Rust 读取接口已接通。用户已明确授权在 peiqian 共享 `football_data` 通过 SQLx 迁移新建此表，迁移账本已记录 `20261013000100`，无需再次建表或迁移；后续经用户授权已将生产应用部署并重启为 `football-insight-service-backend-rs:5827a95`，线上 GET 已验证真实两篇正文和四川话音频。Python 在该表原子发布，local233 的 18092 后端经现有私有隧道读取。CPU 模型安装和手动运行入口见 [local233 今日话题说明](../sina_csl_scraper/docs/topics-local233.md)。

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

payload 为上面 API 数据（最多 5 项）。id 根据球队、日期、材料指纹、生成版本稳定计算。重复运行同材料优先复用本地批次，避免再次 LLM/TTS；重试仅补失败音频。写入通过单事务 upsert，Rust 按文章发布时间从全部球队批次读取最近文章，重复 ID 取最新批次版本，首页 limit 限制 1..5。空生成/采集失败不能覆盖上一批有效文章。LLM 生成与 TTS 不在 Rust 请求链路中运行。

## 实现与验证计划

1. Python 的 topics 包分为 models、sources、generation、tts、pipeline、repository、cli，复用 MinioAssetTarget。支持配置 RSS/Atom 与明确文章 URL，全文抓取、来源时间校验、去重与近期过滤；不能拿搜索摘要冒充全文。
2. LLM 使用可配置兼容 chat-completions 的接口，严格校验 JSON、来源引用、球队范围与 5 项上限。本地已下载的是 TTS 模型，不能用它生成新闻正文。
3. 本地 Qwen 使用 Eric 四川话音色，分段生成 WAV，ffmpeg 转 AAC/m4a，验证非静音并复用内容哈希缓存；音频缺失时文章可发布，audio 标记 failed/pending。
4. 小程序独立获取话题，展示卡片、全文与来源，保留独立音频播放器和生命周期清理。首页原有榜单等其它区域不受话题接口故障影响。
5. Python 用 pytest 覆盖来源时间、去重、LLM 引用校验、缓存/失败和语音命令；实际调用本地 Qwen 验证一段四川话。小程序跑类型、边界检查和双端构建。Rust 迁移与读接口由另一位 agent 实现。
