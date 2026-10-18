# 赛季总结 API

日期：2026-10-18。全部接口为新增 C 端接口，使用现有 `Authorization: Bearer <access_token>`；无会员档位要求。不接受客户端 user_id。

## GET /api/v1/season-recap/{season}

season：2026 至当前 UTC 年份。默认赛季由客户端固定为2026，不随赛季结束失效。

返回200：

```json
{"season":2026,"matches":[{"match_id":1,"match_date":"2026-10-01","opponent":"北京国安","is_home":true}],"selected_match_ids":[1],"home_one_way_km":12.25,"summary":{"away_km":0,"home_km":25,"total_km":25,"matches":1,"home_matches":1,"away_matches":0,"cities":1}}
```

- matches仅包含本赛季、source_active=true、status='3'、球队名称精确等于成都蓉城的一线队比赛；不纳入成都蓉城希拉谷。
- selected_match_ids仅返回仍在可选列表中的保存记录；取消、未完赛等历史勾选留在存储但不计里程。
- home_one_way_km每用户一个值；没有填写时null。home_km只有距离已知时为整数，否则null；无主场且没有距离可正常生成客场海报。
- 城市数按城市名称去重，选主场计成都，上海/青岛各两个球队仍各算一个城市。
- 只读REPEATABLE READ事务，不使用FOR SHARE，不锁住抓取器的比赛行。

## PUT /api/v1/season-recap/{season}

```json
{"selected_match_ids":[1,2],"home_one_way_km":12.25}
```

- selected_match_ids必填，最多60项，重复ID只记一次，空数组清空该赛季勾选；不改其他用户/赛季。
- home_one_way_km可选，数值范围0–500且必须有限。**省略或null保留已存距离**；0是合法值，可明确设置为0，不提供清除距离操作。小程序输入限制最多两位小数。
- 距离偏好upsert串行化同用户保存；当前比赛行只在保存事务中FOR SHARE。应用层回调在锁内验证当前比赛与里程，失败整体回滚；验证、替换勾选、保存距离同一事务提交。
- 如未保存过主场距离而勾选主场，返回400「请填写主场单边距离」。非法距离/比赛/未知城市/赛季返回400和固定中文提示。
- 成功200，响应与GET一致。输入8KB上限；JSON类型/缺字段由Axum返回422，超上限413。持久化不可用503，不暴露SQL原文。

## POST /api/v1/season-recap/{season}/code

无请求体。200返回：

```json
{"image_base64":"...","mime_type":"image/png"}
```

mime_type可能是image/png或image/jpeg，客户端按实际格式落临时文件/绘制；微信错误JSON不作为图片。

- 服务端用既有小程序appid/secret调用stable_token（force_refresh=false）与getwxacodeunlimit；没有新增配置键，不发送消息。
- 固定page=`pages/season-recap/index`，scene=`s=share&c=recap2026`（年份随season），与已上线来源解析器匹配；解析后src=share/cid=recap2026，无任何账号标识。
- 使用release、check_path=false（生成阶段容许新页面未发布，不代表旧小程序能打开）。**新小程序审核通过并正式发布后才对外使用海报入口**。
- 每赛季成功图片进程内缓存24小时；失败冷却30秒；并发合并；单个微信请求超时5秒。生产微信未联调时返回503并由页面显示重试提示，不用伪二维码兜底。

三个接口未登录/无效或过期JWT返回401「请先登录」。旧接口路径、请求、响应、错误文案与状态码均未修改。

## 里程口径

地球半径6371km，Haversine公式；客场每场成都至对手主场城市中心的大圆距离×2，单场四舍五入公里后求和；主场为单边距离×2×主场场次，合计四舍五入；总里程为两个整数小计相加。具体城市中心来源与16队映射见根目录验证报告。
