# 最佳盯盘时段 API

## GET /api/v1/ticket-watch/best-windows

公开只读，无请求体/查询参数。可选`Authorization: Bearer <user JWT>`，沿用current-board的可选鉴权；无效/未登录凭证按preview，不使用Admin JWT。有效V6及以上full；身份查询失败preview。

接口始终HTTP200，JSON字段：

|字段|类型|语义|
|---|---|---|
|status|string|ok / insufficient（有效样本少于3）/ unavailable（初始聚合尚未成功）|
|access|string|full / preview|
|sample_size|number|最终有效样本数|
|generated_at|string|成功聚合时间，RFC3339；刷新失败保留旧值|
|top_windows|array|full最多3项，preview最多1项；非ok为空|
|hours|array|full+ok恒24项，preview或非ok为空|

`top_windows`项为`{start_hour,end_hour,avg_events,hit_matches}`，**preview 中 `start_hour`、`end_hour` 恒为 null**（具体时段是付费内容，不下发）；`hours`项为`{hour,avg_events,share,hit_matches}`。按北京时间小时分桶，平均保留1位小数，占比保留3位小数。相邻小时不合并，峰值按平均降序、小时升序稳定取前三。

示例（preview）：

```json
{"status":"ok","access":"preview","sample_size":8,"generated_at":"2026-10-16T13:00:00+00:00","top_windows":[{"start_hour":null,"end_hour":null,"avg_events":38.4,"hit_matches":7}],"hours":[]}
```

后台选已结束、include_in_reflux_stats、一线队精确主场、有效开售时间的最近8场；起点为开售+10分钟，404在固定8场内跳过，不补老场；少于20事件丢弃，少于3场insufficient。monitor的其他错误保留上次成功结果，初始无结果unavailable；失败1分钟重试，成功10分钟刷新。前台不访问monitor，不计算窗口SQL。

上游协议为`GET /api/match/restock-profile/{match_id}?since=<RFC3339>`，since百分号编码；文件及HTTP均要求schema_version=2、stat_unit=seat_restock_event、seats_with_events和24个hour/events对象。座位即block_name；读端核对ID、起点、类别和桶合计。旧格式404，合法版本损坏或计数错误为刷新失败。monitor发布和离线首次生成步骤见其docs/restock-profile-offline.md。

既有接口的路径、响应、状态码保持不变；无需数据库迁移。
