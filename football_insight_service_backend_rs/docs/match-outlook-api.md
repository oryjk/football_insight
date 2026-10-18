# 赛前数据倾向（只读）

公开数据接口，沿用 `/api/v1/live/**` 的 JSON 响应和无 JWT 约定。不计算概率，不自动触发生成，不修改已有接口；小程序展示层另外遵守登录和审核态门控。当前没有额外的 HTTP 缓存层，冻结状态由数据库当前时间判断。

## 单场

`GET /api/v1/live/match-outlook?match_id=123`

`match_id` 是新浪赛程的比赛标识，不是抢票业务 ID。正整数必填，非法值返回 400。

```json
{
  "status": "ready",
  "outlook": {
    "match_id": 123,
    "home_win": 0.5,
    "draw": 0.25,
    "away_win": 0.25,
    "tau": 0.7,
    "model_version": "elo-v2",
    "generated_at": "2026-10-18T05:00:00Z",
    "kickoff_at": "2026-10-25T11:35:00Z",
    "frozen": false,
    "outcome": null,
    "schedule_changed": false
  }
}
```

无记录、赛程已失效或新表尚未迁移时返回 200：`{"status":"unavailable","outlook":null}`。`outcome` 为主胜 0、平 1、客胜 2，未计分为 null。冻结后改期只标记 `schedule_changed`，保留原概率与时间。

## 公开计分战绩

`GET /api/v1/live/match-outlook/record?season=2026`

`season` 必填，范围 2014–2100。只统计当前有效赛程中有真实赛前发布且已计分的记录；回填历史比分不会补造历史公开战绩。

```json
{
  "season": 2026,
  "matches": 0,
  "hits": 0,
  "accuracy": null,
  "rps_model": null,
  "rps_baseline": null,
  "rps_improvement": null
}
```

有记录时 `accuracy` 为 0–1，RPS 是 Python 已存分数的平均值；`rps_improvement = rps_baseline - rps_model`，负数必须如实展示。最大概率并列按主胜、平、客胜顺序取第一个。表缺失返回上述零场/空指标，不伪造命中率；其他读取故障返回 503，不暴露数据库错误。

## 数据生成与上线

迁移：`20261018140000_add_match_outlooks.sql`，仅新增两张表与索引，不增加防御性 UPDATE 触发器；负责人审查后在 peiqian 手动执行。抓取器命令、断点缓存和显式写入开关见 [scraper README](../../sina_csl_scraper/README.md)。完整验证与限制见 [验证报告](../../docs/match-outlook-validation.md)。
